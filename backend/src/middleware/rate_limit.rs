use actix_web::{dev::{ServiceRequest, ServiceResponse, Transform, Service}, Error, HttpResponse, web};
use futures::future::{ok, Ready, LocalBoxFuture};
use std::task::{Context, Poll};

use crate::services::streaming_service;

/// Rate limiting middleware с реальной интеграцией Redis
/// Ограничивает количество запросов от одного IP/пользователя
pub struct RateLimiter {
    pub max_requests: u32,
    pub window_seconds: u64,
    pub key_prefix: String,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_seconds: u64, key_prefix: &str) -> Self {
        RateLimiter {
            max_requests,
            window_seconds,
            key_prefix: key_prefix.to_string(),
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        RateLimiter {
            max_requests: 100,
            window_seconds: 60,
            key_prefix: "api".to_string(),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RateLimiterService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RateLimiterService {
            service,
            max_requests: self.max_requests,
            window_seconds: self.window_seconds,
            key_prefix: self.key_prefix.clone(),
        })
    }
}

pub struct RateLimiterService<S> {
    service: S,
    max_requests: u32,
    window_seconds: u64,
    key_prefix: String,
}

impl<S, B> Service<ServiceRequest> for RateLimiterService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Получение IP клиента
        let client_ip = req.peer_addr()
            .map(|addr| addr.ip().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        // Получение user_id из JWT (если есть)
        let user_id = req.extensions()
            .get::<crate::models::user::JwtClaims>()
            .map(|claims| claims.sub.to_string())
            .unwrap_or_else(|| client_ip.clone());

        let rate_key = format!("rate_limit:{}:{}", self.key_prefix, user_id);
        let max_requests = self.max_requests;
        let window_seconds = self.window_seconds;

        // Получение Redis клиента из app data
        let redis = req.app_data::<web::Data<redis::Client>>()
            .cloned();

        let fut = self.service.call(req);

        Box::pin(async move {
            // Реальная проверка rate limit через Redis
            if let Some(redis_client) = redis {
                match streaming_service::check_rate_limit(
                    &redis_client,
                    &rate_key,
                    max_requests,
                    window_seconds,
                ).await {
                    Ok(_) => {
                        // Rate limit не превышен, продолжаем
                        let res = fut.await?;
                        Ok(res)
                    }
                    Err(crate::errors::AppError::RateLimitExceeded) => {
                        // Rate limit превышен
                        let response = HttpResponse::TooManyRequests()
                            .insert_header(("Retry-After", window_seconds.to_string()))
                            .json(serde_json::json!({
                                "error": "rate_limit_exceeded",
                                "message": "Превышен лимит запросов. Попробуйте позже.",
                                "retry_after_seconds": window_seconds,
                            }));
                        Err(actix_web::error::ErrorTooManyRequests(response))
                    }
                    Err(e) => {
                        // Ошибка Redis — логируем, но пропускаем запрос (fail-open)
                        log::warn!("Rate limit check failed: {}", e);
                        let res = fut.await?;
                        Ok(res)
                    }
                }
            } else {
                // Redis не настроен — пропускаем (fail-open)
                log::warn!("Redis client not found, rate limiting disabled");
                let res = fut.await?;
                Ok(res)
            }
        })
    }
}

/// Предустановленные rate limiter'ы для разных типов эндпоинтов
pub mod presets {
    use super::RateLimiter;

    /// Для стриминга медиа — строгий лимит
    pub fn media_stream() -> RateLimiter {
        RateLimiter::new(30, 60, "media")
    }

    /// Для API — стандартный лимит
    pub fn api_default() -> RateLimiter {
        RateLimiter::new(100, 60, "api")
    }

    /// Для аутентификации — строгий лимит (защита от брутфорса)
    pub fn auth() -> RateLimiter {
        RateLimiter::new(10, 300, "auth")
    }

    /// Для webhook'ов — высокий лимит
    pub fn webhook() -> RateLimiter {
        RateLimiter::new(1000, 60, "webhook")
    }
}
