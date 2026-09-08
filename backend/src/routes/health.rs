use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use serde::Serialize;
use chrono::Utc;

/// Конфигурация health check роутов
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/health")
            .route("", web::get().to(health_check))
            .route("/ready", web::get().to(readiness_check))
            .route("/live", web::get().to(liveness_check))
            .route("/metrics", web::get().to(metrics_endpoint))
    );
}

/// GET /health
/// Полная проверка здоровья сервиса
async fn health_check(pool: web::Data<PgPool>) -> HttpResponse {
    let start = std::time::Instant::now();
    
    // Проверка подключения к БД
    let db_healthy = sqlx::query("SELECT 1")
        .execute(pool.get_ref())
        .await
        .is_ok();
    
    let db_latency_ms = start.elapsed().as_millis();
    
    let status = if db_healthy { "healthy" } else { "unhealthy" };
    let status_code = if db_healthy { 200 } else { 503 };
    
    let response = HealthResponse {
        status: status.to_string(),
        timestamp: Utc::now().to_rfc3339(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        checks: HealthChecks {
            database: ComponentHealth {
                status: if db_healthy { "up" } else { "down" }.to_string(),
                latency_ms: db_latency_ms as u64,
            },
        },
    };
    
    if db_healthy {
        HttpResponse::Ok().json(response)
    } else {
        HttpResponse::ServiceUnavailable().json(response)
    }
}

/// GET /health/ready
/// Проверка готовности принимать запросы (Kubernetes readiness probe)
async fn readiness_check(pool: web::Data<PgPool>) -> HttpResponse {
    // Проверка подключения к БД
    let db_ready = sqlx::query("SELECT 1")
        .execute(pool.get_ref())
        .await
        .is_ok();
    
    if db_ready {
        HttpResponse::Ok().json(serde_json::json!({
            "status": "ready",
            "timestamp": Utc::now().to_rfc3339(),
        }))
    } else {
        HttpResponse::ServiceUnavailable().json(serde_json::json!({
            "status": "not_ready",
            "reason": "database_unavailable",
            "timestamp": Utc::now().to_rfc3339(),
        }))
    }
}

/// GET /health/live
/// Проверка живости сервиса (Kubernetes liveness probe)
async fn liveness_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "alive",
        "timestamp": Utc::now().to_rfc3339(),
    }))
}

/// GET /health/metrics
/// Экспорт метрик в формате Prometheus
async fn metrics_endpoint() -> HttpResponse {
    let metrics = crate::telemetry::export_metrics().await;
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4; charset=utf-8")
        .body(metrics)
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    timestamp: String,
    version: String,
    checks: HealthChecks,
}

#[derive(Serialize)]
struct HealthChecks {
    database: ComponentHealth,
}

#[derive(Serialize)]
struct ComponentHealth {
    status: String,
    latency_ms: u64,
}
