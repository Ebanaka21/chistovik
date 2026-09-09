use sqlx::PgPool;
use uuid::Uuid;
use chrono::{Utc, Duration};
use serde::Serialize;

/// Сервис квот и лимитов для пользователей
pub struct QuotaService {
    pool: PgPool,
}

impl QuotaService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Проверка квоты на загрузку контента
    /// ВАЖНО: Проверка должна быть ДО записи файла, а не после
    pub async fn check_upload_quota(&self, user_id: Uuid, file_size: u64) -> Result<QuotaCheck, QuotaError> {
        // Получаем текущее использование
        let usage = self.get_usage(user_id).await?;
        
        // Получаем лимиты пользователя (зависят от тарифа)
        let limits = self.get_user_limits(user_id).await?;

        // Проверяем лимиты
        if usage.storage_bytes + file_size > limits.max_storage_bytes {
            return Err(QuotaError::StorageExceeded {
                used: usage.storage_bytes,
                limit: limits.max_storage_bytes,
            });
        }

        if usage.contents_count >= limits.max_contents {
            return Err(QuotaError::ContentsCountExceeded {
                used: usage.contents_count,
                limit: limits.max_contents,
            });
        }

        Ok(QuotaCheck {
            allowed: true,
            remaining_storage: limits.max_storage_bytes - usage.storage_bytes,
            remaining_contents: limits.max_contents - usage.contents_count,
        })
    }

    /// Атомарное обновление квоты (в транзакции с INSERT файла)
    /// Используется для предотвращения race condition
    pub async fn update_quota_atomically(
        &self,
        author_id: Uuid,
        file_size: i64,
    ) -> Result<(), QuotaError> {
        // Транзакция для атомарного обновления
        let mut tx = self.pool.begin()
            .await
            .map_err(|e| QuotaError::Database(e.to_string()))?;

        // Блокируем строку автора (FOR UPDATE)
        sqlx::query(
            "SELECT id FROM authors WHERE id = $1 FOR UPDATE"
        )
        .bind(author_id)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| QuotaError::Database(e.to_string()))?;

        // Атомарно увеличиваем total_revenue_kopecks (используем как storage_bytes)
        sqlx::query(
            "UPDATE authors SET total_revenue_kopecks = total_revenue_kopecks + $1 WHERE id = $2"
        )
        .bind(file_size)
        .bind(author_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| QuotaError::Database(e.to_string()))?;

        // Коммитим транзакцию
        tx.commit()
            .await
            .map_err(|e| QuotaError::Database(e.to_string()))?;

        Ok(())
    }

    /// Получение текущего использования
    pub async fn get_usage(&self, user_id: Uuid) -> Result<Usage, QuotaError> {
        let author_id: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM authors WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| QuotaError::Database(e.to_string()))?;

        let author_id = author_id.ok_or(QuotaError::AuthorNotFound)?;

        let storage_bytes: i64 = sqlx::query_scalar(
            "SELECT COALESCE(SUM(file_size_bytes), 0) FROM contents WHERE author_id = $1"
        )
        .bind(author_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| QuotaError::Database(e.to_string()))?;

        let contents_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM contents WHERE author_id = $1"
        )
        .bind(author_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| QuotaError::Database(e.to_string()))?;

        let subscribers_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM subscriptions WHERE author_id = $1 AND status = 'active'"
        )
        .bind(author_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| QuotaError::Database(e.to_string()))?;

        Ok(Usage {
            storage_bytes: storage_bytes as u64,
            contents_count: contents_count as u64,
            subscribers_count: subscribers_count as u64,
        })
    }

    /// Получение лимитов пользователя
    pub async fn get_user_limits(&self, user_id: Uuid) -> Result<Limits, QuotaError> {
        // В реальности — из БД или конфигурации
        // Пока — хардкод для примера
        Ok(Limits {
            max_storage_bytes: 10 * 1024 * 1024 * 1024, // 10 GB
            max_contents: 1000,
            max_file_size: 2 * 1024 * 1024 * 1024, // 2 GB
            max_daily_uploads: 50,
        })
    }

    /// Проверка дневного лимита загрузок
    pub async fn check_daily_upload_limit(&self, user_id: Uuid) -> Result<bool, QuotaError> {
        let author_id: Option<Uuid> = sqlx::query_scalar(
            "SELECT id FROM authors WHERE user_id = $1"
        )
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| QuotaError::Database(e.to_string()))?;

        let author_id = author_id.ok_or(QuotaError::AuthorNotFound)?;

        let today_uploads: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM contents WHERE author_id = $1 AND created_at >= date_trunc('day', NOW())"
        )
        .bind(author_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| QuotaError::Database(e.to_string()))?;

        let limits = self.get_user_limits(user_id).await?;

        Ok(today_uploads < limits.max_daily_uploads as i64)
    }
}

#[derive(Debug, Serialize)]
pub struct QuotaCheck {
    pub allowed: bool,
    pub remaining_storage: u64,
    pub remaining_contents: u64,
}

#[derive(Debug, Serialize)]
pub struct Usage {
    pub storage_bytes: u64,
    pub contents_count: u64,
    pub subscribers_count: u64,
}

#[derive(Debug, Serialize)]
pub struct Limits {
    pub max_storage_bytes: u64,
    pub max_contents: u64,
    pub max_file_size: u64,
    pub max_daily_uploads: u64,
}

#[derive(Debug)]
pub enum QuotaError {
    StorageExceeded { used: u64, limit: u64 },
    ContentsCountExceeded { used: u64, limit: u64 },
    DailyLimitExceeded,
    AuthorNotFound,
    Database(String),
}

impl std::fmt::Display for QuotaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QuotaError::StorageExceeded { used, limit } => {
                write!(f, "Storage quota exceeded: {} / {} bytes", used, limit)
            }
            QuotaError::ContentsCountExceeded { used, limit } => {
                write!(f, "Contents count exceeded: {} / {}", used, limit)
            }
            QuotaError::DailyLimitExceeded => write!(f, "Daily upload limit exceeded"),
            QuotaError::AuthorNotFound => write!(f, "Author not found"),
            QuotaError::Database(e) => write!(f, "Database error: {}", e),
        }
    }
}

impl std::error::Error for QuotaError {}
