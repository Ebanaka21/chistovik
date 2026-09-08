use actix_web::{web, App, HttpServer, middleware};
use actix_cors::Cors;
use sqlx::postgres::PgPoolOptions;
use std::env;
use dotenv::dotenv;

mod config;
mod models;
mod routes;
mod middleware as app_middleware;
mod services;
mod errors;
mod telemetry;

use config::Config;

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
            // FIX: Metrics middleware (должен быть до security для корректного подсчёта)
            .wrap(telemetry::middleware::MetricsMiddleware)
            // FIX: Security headers middleware
            .wrap(app_middleware::security::SecurityHeaders)
            .app_data(config_data.clone())
            .app_data(pool_data.clone())
            .app_data(redis_data.clone())
            // Health check routes
            .configure(routes::health::configure)
            // Public routes
            .configure(routes::auth::configure)
            .configure(routes::showcase::configure)
            // Protected routes
            .service(
                web::scope("/api/v1")
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
