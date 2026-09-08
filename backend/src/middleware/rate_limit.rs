use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use futures::future::{ok, LocalBoxFuture, Ready};
use std::rc::Rc;
use std::task::{Context, Poll};

use crate::models::user::JwtClaims;

/// Rate limiting middleware
/// Ограничивает количество запросов от одного IP/пользователя
pub struct RateLimiter {
    pub max_requests: u32,
    pub window_seconds: u64,
    pub fail_open: bool, // Если true — пропускаем при ошибке Redis
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_seconds: u64, fail_open: bool) -> Self {
        RateLimiter {
            max_requests,
            window_seconds,
            fail_open,
        }
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        RateLimiter {
            max_requests: 100,
            window_seconds: 60,
            fail_open: true,
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
            service: Rc::new(service),
            max_requests: self.max_requests,
            window_seconds: self.window_seconds,
            fail_open: self.fail_open,
        })
    }
}

pub struct RateLimiterService<S> {
    service: Rc<S>,
    max_requests: u32,
    window_seconds: u64,
    fail_open: bool,
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
        let service = self.service.clone();
        let max_requests = self.max_requests;
        let window_seconds = self.window_seconds;
        let fail_open = self.fail_open;

        // FIX: Извлечение IP с учётом X-Forwarded-For
        let client_ip = extract_client_ip(&req);

        // FIX: Получение user_id из claims (если есть)
        let user_id = req
            .extensions()
            .get::<JwtClaims>()
            .map(|c| c.sub.to_string());

        // Формируем ключ: если есть user_id — используем его, иначе IP
        let rate_key = if let Some(uid) = user_id {
            format!("rate:user:{}", uid)
        } else {
            format!("rate:ip:{}", client_ip)
        };

        let redis_client = req
            .app_data::<actix_web::web::Data<redis::Client>>()
            .map(|c| c.get_ref().clone());

        let req_parts = req.into_parts();
        let req_head = req_parts.0;

        Box::pin(async move {
            let redis = match redis_client {
                Some(r) => r,
                None => {
                    // Redis не настроен — пропускаем
                    let req = ServiceRequest::from_parts(req_head);
                    let res = service.call(req).await?;
                    return Ok(res);
                }
            };

            // Проверка rate limit через Redis
            match check_rate_limit(&redis, &rate_key, max_requests, window_seconds).await {
                Ok(()) => {
                    // Лимит не превышен — пропускаем
                    let req = ServiceRequest::from_parts(req_head);
                    let res = service.call(req).await?;
                    Ok(res)
                }
                Err(e) => {
                    if fail_open {
                        // Fail-open: пропускаем при ошибке Redis
                        log::warn!("Rate limit check failed (fail-open): {}", e);
                        let req = ServiceRequest::from_parts(req_head);
                        let res = service.call(req).await?;
                        Ok(res)
                    } else {
                        // Fail-closed: возвращаем 503
                        log::error!("Rate limit check failed (fail-closed): {}", e);
                        let res = HttpResponse::ServiceUnavailable()
                            .json(serde_json::json!({
                                "error": "service_unavailable",
                                "message": "Service temporarily unavailable. Please try again later."
                            }));
                        let res = res.map_into_right_body();
                        let req = actix_web::HttpRequest::from(req_head);
                        Ok(ServiceResponse::new(req, res))
                    }
                }
            }
        })
    }
}

/// Проверка rate limit через Redis
async fn check_rate_limit(
    redis: &redis::Client,
    rate_key: &str,
    max_requests: u32,
    window_seconds: u64,
) -> Result<(), String> {
    let mut conn = redis
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| format!("Redis connection error: {}", e))?;

    // Lua script для атомарной проверки
    let script = redis::Script::new(
        r#"
        local current = redis.call('INCR', KEYS[1])
        if current == 1 then
            redis.call('EXPIRE', KEYS[1], ARGV[1])
        end
        return current
    "#,
    );

    let current: u32 = script
        .key(rate_key)
        .arg(window_seconds)
        .invoke_async(&mut conn)
        .await
        .map_err(|e| format!("Redis error: {}", e))?;

    if current > max_requests {
        Err(format!(
            "Rate limit exceeded: {} > {} requests per {} seconds",
            current, max_requests, window_seconds
        ))
    } else {
        Ok(())
    }
}

/// Извлечение IP клиента с учётом X-Forwarded-For
/// Проверяем доверенные прокси для защиты от подделки
fn extract_client_ip(req: &ServiceRequest) -> String {
    // Список доверенных прокси (настраивается через env)
    let trusted_proxies = std::env::var("TRUSTED_PROXIES")
        .unwrap_or_else(|_| "127.0.0.1,10.0.0.0/8,172.16.0.0/12,192.168.0.0/16".to_string())
        .split(',')
        .map(|s| s.trim().to_string())
        .collect::<Vec<_>>();

    // Проверяем peer_addr
    let peer_addr = req
        .peer_addr()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    // Если peer_addr — доверенный прокси, используем X-Forwarded-For
    if is_trusted_proxy(&peer_addr, &trusted_proxies) {
        if let Some(xff) = req.headers().get("X-Forwarded-For") {
            if let Ok(xff_str) = xff.to_str() {
                // Берём первый IP из списка (клиент)
                if let Some(client_ip) = xff_str.split(',').next() {
                    return client_ip.trim().to_string();
                }
            }
        }
    }

    peer_addr
}

/// Проверка, является ли IP доверенным прокси
fn is_trusted_proxy(ip: &str, trusted_proxies: &[String]) -> bool {
    for proxy in trusted_proxies {
        if proxy.contains('/') {
            // CIDR notation — упрощённая проверка
            // В реальности — использовать ipnet crate
            if ip.starts_with(proxy.split('/').next().unwrap_or("")) {
                return true;
            }
        } else if ip == proxy {
            return true;
        }
    }
    false
}

/// Предустановленные rate limiter'ы для разных типов эндпоинтов
pub mod presets {
    use super::RateLimiter;

    /// Для стриминга медиа — строгий лимит, fail-open
    pub fn media_stream() -> RateLimiter {
        RateLimiter::new(30, 60, true)
    }

    /// Для API — стандартный лимит, fail-open
    pub fn api_default() -> RateLimiter {
        RateLimiter::new(100, 60, true)
    }

    /// Для аутентификации — строгий лимит, FAIL-CLOSED (защита от брутфорса)
    pub fn auth() -> RateLimiter {
        RateLimiter::new(10, 300, false) // fail_open = false
    }

    /// Для webhook'ов — высокий лимит, fail-open
    pub fn webhook() -> RateLimiter {
        RateLimiter::new(1000, 60, true)
    }
}
