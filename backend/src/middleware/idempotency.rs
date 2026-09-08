use actix_web::{dev::{ServiceRequest, ServiceResponse, Transform, Service}, Error, HttpResponse, web};
use futures::future::{ok, Ready, LocalBoxFuture};
use std::task::{Context, Poll};
use uuid::Uuid;
use redis::AsyncCommands;
use chrono::{Utc, Duration};

/// Idempotency middleware для защиты от дублей
/// Работает только для POST/PUT/PATCH запросов с заголовком Idempotency-Key
pub struct IdempotencyMiddleware;

impl<S, B> Transform<S, ServiceRequest> for IdempotencyMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = IdempotencyService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(IdempotencyService { service })
    }
}

pub struct IdempotencyService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for IdempotencyService<S>
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
        // Проверяем наличие Idempotency-Key
        let idempotency_key = req.headers()
            .get("Idempotency-Key")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // Применяем только к mutation-методам
        let method = req.method().clone();
        let is_mutation = method == actix_web::http::Method::POST 
            || method == actix_web::http::Method::PUT 
            || method == actix_web::http::Method::PATCH;

        let redis_client = req.app_data::<web::Data<redis::Client>>().cloned();
        let pool = req.app_data::<web::Data<sqlx::PgPool>>().cloned();

        let fut = self.service.call(req);

        Box::pin(async move {
            // Если нет ключа или не mutation — пропускаем
            if !is_mutation || idempotency_key.is_none() {
                return fut.await;
            }

            let key = idempotency_key.unwrap();
            let user_id = extract_user_id_from_request(&key); // Извлекаем из JWT

            // Создаём уникальный ключ для Redis
            let redis_key = format!("idempotency:{}:{}", user_id, key);

            // Проверяем в Redis
            if let Some(redis) = redis_client {
                let mut conn = redis.get_multiplexed_async_connection().await
                    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Redis error: {}", e)))?;

                // Пытаемся получить существующий ответ
                let cached_response: Option<String> = conn.get(&redis_key).await
                    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Redis get error: {}", e)))?;

                if let Some(response_json) = cached_response {
                    // Возвращаем кэшированный ответ
                    log::info!("Idempotency hit for key: {}", key);
                    let response: serde_json::Value = serde_json::from_str(&response_json)
                        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("JSON parse error: {}", e)))?;
                    
                    // Создаём HttpResponse из кэша
                    return Ok(actix_web::HttpResponse::Ok().json(response).map_into_right_body());
                }

                // Устанавливаем lock на 60 секунд (защита от race condition)
                let lock_acquired: bool = redis::cmd("SET")
                    .arg(&format!("{}:lock", redis_key))
                    .arg("1")
                    .arg("NX")
                    .arg("EX")
                    .arg(60)
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Redis lock error: {}", e)))?;

                if !lock_acquired {
                    // Уже выполняется запрос с этим ключом
                    return Err(actix_web::error::ErrorConflict("Request already in progress"));
                }
            }

            // Выполняем запрос
            let res = fut.await?;

            // Кэшируем ответ в Redis (если успешно)
            if res.status().is_success() {
                if let (Some(redis), Some(pool)) = (redis_client, pool) {
                    let mut conn = redis.get_multiplexed_async_connection().await.ok();
                    
                    if let Some(mut conn) = conn {
                        // Извлекаем body из response (упрощённо)
                        let response_body = serde_json::json!({
                            "status": res.status().as_u16(),
                            "cached_at": Utc::now().to_rfc3339()
                        });

                        // Сохраняем на 24 часа
                        let _: () = conn.set_ex(
                            &redis_key,
                            response_body.to_string(),
                            86400
                        ).await.ok();

                        // Удаляем lock
                        let _: () = conn.del(&format!("{}:lock", redis_key)).await.ok();
                    }
                }
            }

            Ok(res)
        })
    }
}

/// Извлечение user_id из idempotency key (упрощённо)
fn extract_user_id_from_request(key: &str) -> String {
    // В реальности извлекаем из JWT токена в request extensions
    format!("user_{}", &key[..8.min(key.len())])
}

/// Сервис для работы с idempotency keys в БД (fallback если Redis недоступен)
pub struct IdempotencyService {
    pool: sqlx::PgPool,
}

impl IdempotencyService {
    pub fn new(pool: sqlx::PgPool) -> Self {
        Self { pool }
    }

    /// Проверка существования idempotency key
    pub async fn check_key(&self, user_id: Uuid, key: &str) -> Result<Option<serde_json::Value>, sqlx::Error> {
        let result: Option<(serde_json::Value,)> = sqlx::query_as(
            "SELECT response_body FROM idempotency_keys WHERE user_id = $1 AND key = $2 AND created_at > NOW() - INTERVAL '24 hours'"
        )
        .bind(user_id)
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;

        Ok(result.map(|(body,)| body))
    }

    /// Сохранение idempotency key
    pub async fn save_key(&self, user_id: Uuid, key: &str, response: serde_json::Value) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO idempotency_keys (id, user_id, key, response_body, created_at) VALUES ($1, $2, $3, $4, NOW())"
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(key)
        .bind(response)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
