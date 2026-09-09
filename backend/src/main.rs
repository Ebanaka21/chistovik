use actix_web::{web, App, HttpServer, middleware};
use actix_cors::Cors;
use sqlx::postgres::PgPoolOptions;
use std::env;
use std::sync::Arc;
use dotenv::dotenv;

mod config;
mod models;
mod routes;
mod middleware as app_middleware;
mod services;
mod errors;
mod telemetry;

use config::Config;
use app_middleware::graceful_degradation::{SystemMonitor, DegradationConfig, ConnectionCounter};

#[cfg(test)]
mod tests;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    
    // Инициализация telemetry
    if std::env::var("OTEL_ENABLED").unwrap_or_default() == "true" {
        let otlp_endpoint = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT")
            .unwrap_or_else(|_| "http://localhost:4317".to_string());
        
        match telemetry::init_tracing("chistovik-backend", &otlp_endpoint) {
            Ok(_tracer) => {
                log::info!("OpenTelemetry tracing initialized");
            }
            Err(e) => {
                log::warn!("Failed to initialize OpenTelemetry: {}", e);
            }
        }
    }
    
    // Инициализация Prometheus метрик
    if let Err(e) = telemetry::init_metrics() {
        log::warn!("Failed to initialize metrics: {}", e);
    }
    
    // Регистрация метрик
    telemetry::metrics::register_all().await;
    
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let config = Config::from_env();

    // Database connection pool
    let pool = PgPoolOptions::new()
        .max_connections(config.database_max_connections)
        .connect(&config.database_url)
        .await
        .expect("Failed to create database pool");

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    log::info!("Starting server at http://{}:{}", config.host, config.port);

    // Redis connection
    let redis_client = redis::Client::open(config.redis_url.clone())
        .expect("Failed to create Redis client");

    let config_data = web::Data::new(config.clone());
    let pool_data = web::Data::new(pool.clone());
    let redis_data = web::Data::new(redis_client);

    // System monitor для graceful degradation
    let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
    let degradation_config = DegradationConfig::default();
    let mut system_monitor = SystemMonitor::new(degradation_config.clone(), shutdown_rx);
    let system_status = system_monitor.get_status();

    // Запуск system monitor в отдельной задаче
    tokio::spawn(async move {
        system_monitor.run().await;
    });

    // Job queue для FFmpeg задач
    let job_queue = services::job_queue::JobQueue::new(pool.clone(), 4); // Максимум 4 параллельных FFmpeg
    let (job_shutdown_tx, job_shutdown_rx) = tokio::sync::mpsc::channel(1);
    
    // Запуск job worker в отдельной задаче
    let mut job_worker = services::job_queue::JobWorker::new(job_queue, job_shutdown_rx);
    tokio::spawn(async move {
        if let Err(e) = job_worker.run().await {
            log::error!("Job worker error: {}", e);
        }
    });

    // FIX: Graceful shutdown через tokio signal с ожиданием завершения запросов
    let server = HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin(&config.cors_origin)
            .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE"])
            .allowed_headers(vec!["Content-Type", "Authorization"])
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .wrap(middleware::Compress::default())
            // FIX: Connection counter (должен быть снаружи для подсчёта всех подключений)
            .wrap(ConnectionCounter)
            // FIX: Graceful degradation middleware
            .wrap(app_middleware::graceful_degradation::GracefulDegradationMiddleware::new(
                system_status.clone(),
                DegradationConfig::default(),
            ))
            // FIX: Metrics middleware (должен быть до security для корректного подсчёта)
            .wrap(telemetry::middleware::MetricsMiddleware)
            // FIX: Security headers middleware
            .wrap(app_middleware::security::SecurityHeaders)
            .app_data(config_data.clone())
            .app_data(pool_data.clone())
            .app_data(redis_data.clone())
            // Health check routes
            .configure(routes::health::configure)
            // Public routes с rate limiting по IP (fail-open)
            .service(
                web::scope("")
                    .wrap(app_middleware::rate_limit::presets::api_default())
                    .configure(routes::auth::configure)
                    .configure(routes::showcase::configure)
            )
            // Protected routes: Auth → RateLimiter → handlers
            // RateLimiter внутри AuthMiddleware для ключевания по user_id
            .service(
                web::scope("/api/v1")
                    .wrap(app_middleware::rate_limit::presets::api_default())
                    .wrap(app_middleware::auth::AuthMiddleware)
                    .configure(routes::author::configure)
                    .configure(routes::content::configure)
                    .configure(routes::subscription::configure)
                    .configure(routes::payment::configure)
                    .configure(routes::user::configure)
                    .configure(routes::admin::configure)
            )
            // Media streaming routes (separate auth via tokens)
            .configure(routes::streaming::configure)
    })
    .bind(format!("{}:{}", config.host, config.port))?
    .workers(num_cpus::get())
    .shutdown_timeout(30) // Ждём 30 секунд завершения активных запросов
    .run();

    // FIX: Graceful shutdown с ожиданием завершения запросов
    log::info!("Server started. Press Ctrl+C to stop.");
    
    let graceful_server = server;
    
    tokio::select! {
        result = graceful_server => {
            if let Err(e) = result {
                log::error!("Server error: {}", e);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            log::info!("Received Ctrl+C, initiating graceful shutdown...");
            
            // Сигнализируем system monitor о shutdown
            let _ = shutdown_tx.send(true);
            
            // Сигнализируем job worker о shutdown
            let _ = job_shutdown_tx.send(()).await;
            
            log::info!("Waiting for active requests to complete (max 30s)...");
            
            // Даём время завершиться активным запросам
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            
            log::info!("Graceful shutdown complete.");
        }
    }

    // Закрываем подключение к БД
    log::info!("Closing database connections...");
    pool.close().await;
    
    log::info!("Server stopped.");
    Ok(())
}
