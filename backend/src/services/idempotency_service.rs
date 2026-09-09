use redis::AsyncCommands;
use serde::{de::DeserializeOwned, Serialize};
use sqlx::PgPool;
use uuid::Uuid;
use chrono::Utc;

/// Idempotency service — кэширует РЕАЛЬНЫЕ результаты операций
/// Используется как обёртка над бизнес-логикой, а не middleware
pub struct IdempotencyService {
    pool: PgPool,
    redis: redis::Client,
}

impl IdempotencyService {
    pub fn new(pool: PgPool, redis: redis::Client) -> Self {
        Self { pool, redis }
    }

    /// Выполнение идемпотентной операции
    /// Кэширует РЕАЛЬНЫЙ результат (не пустышку!)
    pub async fn execute_idempotent<F, Fut, T>(
        &self,
        user_id: Uuid,
        key: &str,
        operation: F,
    ) -> Result<T, IdempotencyError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T, IdempotencyError>>,
        T: Serialize + DeserializeOwned,
    {
        let redis_key = format!("idemp:{}:{}", user_id, key);
        let lock_key = format!("{}:lock", redis_key);

        let mut conn = self
            .redis
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| IdempotencyError::Redis(e.to_string()))?;

        // 1. Проверяем кэш в Redis
        if let Ok(Some(cached)) = conn.get::<_, Option<String>>(&redis_key).await {
            log::debug!("Idempotency cache hit for key: {}", key);
            return serde_json::from_str(&cached)
                .map_err(|e| IdempotencyError::Deserialization(e.to_string()));
        }

        // 2. Fallback: проверяем в PostgreSQL
        if let Some(pg_cached) = self.check_key_pg(user_id, key).await? {
            log::debug!("Idempotency PostgreSQL cache hit for key: {}", key);
            return serde_json::from_str(&pg_cached)
                .map_err(|e| IdempotencyError::Deserialization(e.to_string()));
        }

        // 3. Acquire lock (NX + EX 120 сек — достаточно для долгих операций)
        let acquired: bool = redis::cmd("SET")
            .arg(&lock_key)
            .arg("1")
            .arg("NX")
            .arg("EX")
            .arg(120)
            .query_async(&mut conn)
            .await
            .map_err(|e| IdempotencyError::Redis(e.to_string()))?;

        if !acquired {
            return Err(IdempotencyError::Conflict(
                "Request already in progress".to_string(),
            ));
        }

        // 4. Выполняем РЕАЛЬНУЮ операцию
        let result = operation().await;

        // 5. Кэшируем РЕАЛЬНЫЙ результат (только успех)
        if let Ok(ref value) = result {
            let serialized = serde_json::to_string(value)
                .map_err(|e| IdempotencyError::Serialization(e.to_string()))?;

            // Сохраняем в Redis (TTL 24 часа)
            let _: Result<(), _> = conn.set_ex(&redis_key, &serialized, 86400).await;

            // Fallback: сохраняем в PostgreSQL
            let _: Result<(), _> = self.save_key_pg(user_id, key, &serialized).await;

            log::info!("Idempotency result cached for key: {}", key);
        }

        // 6. Освобождаем lock
        let _: Result<(), _> = conn.del(&lock_key).await;

        result
    }

    /// Проверка ключа в PostgreSQL (fallback)
    async fn check_key_pg(&self, user_id: Uuid, key: &str) -> Result<Option<String>, IdempotencyError> {
        let result: Option<(String,)> = sqlx::query_as(
            "SELECT response_body FROM idempotency_keys WHERE user_id = $1 AND key = $2 AND created_at > NOW() - INTERVAL '24 hours'"
        )
        .bind(user_id)
        .bind(key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| IdempotencyError::Database(e.to_string()))?;

        Ok(result.map(|(body,)| body))
    }

    /// Сохранение ключа в PostgreSQL (fallback)
    async fn save_key_pg(&self, user_id: Uuid, key: &str, response_body: &str) -> Result<(), IdempotencyError> {
        sqlx::query(
            "INSERT INTO idempotency_keys (id, user_id, key, response_body, created_at) VALUES ($1, $2, $3, $4, NOW()) ON CONFLICT (user_id, key) DO NOTHING"
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(key)
        .bind(response_body)
        .execute(&self.pool)
        .await
        .map_err(|e| IdempotencyError::Database(e.to_string()))?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum IdempotencyError {
    Redis(String),
    Database(String),
    Serialization(String),
    Deserialization(String),
    Conflict(String),
}

impl std::fmt::Display for IdempotencyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdempotencyError::Redis(e) => write!(f, "Redis error: {}", e),
            IdempotencyError::Database(e) => write!(f, "Database error: {}", e),
            IdempotencyError::Serialization(e) => write!(f, "Serialization error: {}", e),
            IdempotencyError::Deserialization(e) => write!(f, "Deserialization error: {}", e),
            IdempotencyError::Conflict(e) => write!(f, "Conflict: {}", e),
        }
    }
}

impl std::error::Error for IdempotencyError {}

impl From<IdempotencyError> for crate::errors::AppError {
    fn from(e: IdempotencyError) -> Self {
        match e {
            IdempotencyError::Conflict(msg) => crate::errors::AppError::ValidationError(msg),
            _ => crate::errors::AppError::InternalError(e.to_string()),
        }
    }
}
