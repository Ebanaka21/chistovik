use sqlx::PgPool;
use uuid::Uuid;
use chrono::{Utc, Duration};
use regex::Regex;
use std::collections::HashMap;

/// Антиспам сервис — фильтрация нежелательного контента
pub struct AntiSpamService {
    pool: PgPool,
    spam_patterns: Vec<Regex>,
    banned_words: Vec<String>,
}

impl AntiSpamService {
    pub fn new(pool: PgPool) -> Self {
        // Паттерны спама
        let spam_patterns = vec![
            Regex::new(r"(?i)(купить|продам|заработай|бесплатно|халява|акция|скидка)\s*\d+").unwrap(),
            Regex::new(r"(?i)(t\.me|telegram\.me)/[a-zA-Z0-9_]+").unwrap(),
            Regex::new(r"(?i)\b(crypto|bitcoin|forex|investment|earn money)\b").unwrap(),
            Regex::new(r"(?i)(click here|перейди|жми)\s*(http|https|t\.me)").unwrap(),
            Regex::new(r"[A-Z]{5,}").unwrap(), // КАПС
            Regex::new(r"(.)\1{5,}").unwrap(), // Повторы символов
        ];

        // Запрещённые слова
        let banned_words = vec![
            "казино".to_string(),
            "ставки".to_string(),
            "порно".to_string(),
            "нарко".to_string(),
        ];

        Self {
            pool,
            spam_patterns,
            banned_words,
        }
    }

    /// Проверка текста на спам
    pub async fn check_content(&self, user_id: Uuid, content: &str, content_type: &str) -> Result<SpamCheckResult, SpamError> {
        let mut score = 0.0;
        let mut reasons = Vec::new();

        // Проверка на паттерны спама
        for pattern in &self.spam_patterns {
            if pattern.is_match(content) {
                score += 0.3;
                reasons.push(format!("Spam pattern detected: {}", pattern.as_str()));
            }
        }

        // Проверка на запрещённые слова
        let content_lower = content.to_lowercase();
        for word in &self.banned_words {
            if content_lower.contains(word) {
                score += 0.5;
                reasons.push(format!("Banned word: {}", word));
            }
        }

        // Проверка на ссылки
        let url_count = content.matches("http").count() + content.matches("www.").count();
        if url_count > 3 {
            score += 0.2;
            reasons.push(format!("Too many URLs: {}", url_count));
        }

        // Проверка на повторяющийся контент
        if self.is_duplicate_content(user_id, content).await? {
            score += 0.4;
            reasons.push("Duplicate content detected".to_string());
        }

        // Проверка скорости постинга
        if self.is_posting_too_fast(user_id).await? {
            score += 0.3;
            reasons.push("Posting too fast".to_string());
        }

        // Проверка длины
        if content.len() < 10 {
            score += 0.1;
            reasons.push("Content too short".to_string());
        }

        let is_spam = score >= 0.7;
        let action = if is_spam {
            SpamAction::Block
        } else if score >= 0.4 {
            SpamAction::Moderate
        } else {
            SpamAction::Allow
        };

        // Логируем проверку
        self.log_check(user_id, content, content_type, score, is_spam).await?;

        // Логируем в audit_logs для отслеживания
        if score >= 0.4 {
            self.log_to_audit(user_id, content_type, score, &reasons).await?;
        }

        Ok(SpamCheckResult {
            is_spam,
            score,
            action,
            reasons,
        })
    }

    /// Проверка на дубликаты
    async fn is_duplicate_content(&self, user_id: Uuid, content: &str) -> Result<bool, SpamError> {
        // Ищем похожий контент от этого пользователя за последние 24 часа
        let similar_count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM spam_checks
            WHERE user_id = $1
            AND content_hash = $2
            AND created_at > NOW() - INTERVAL '24 hours'
            "#
        )
        .bind(user_id)
        .bind(self.hash_content(content))
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SpamError::Database(e.to_string()))?;

        Ok(similar_count > 0)
    }

    /// Проверка скорости постинга
    async fn is_posting_too_fast(&self, user_id: Uuid) -> Result<bool, SpamError> {
        let recent_posts: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM spam_checks WHERE user_id = $1 AND created_at > NOW() - INTERVAL '5 minutes'"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| SpamError::Database(e.to_string()))?;

        Ok(recent_posts > 5) // Больше 5 постов за 5 минут
    }

    /// Хэширование контента для поиска дубликатов
    fn hash_content(&self, content: &str) -> String {
        use sha2::{Sha256, Digest};
        let normalized = content.to_lowercase().split_whitespace().collect::<Vec<_>>().join(" ");
        let hash = Sha256::digest(normalized.as_bytes());
        hex::encode(hash)
    }

    /// Логирование проверки
    async fn log_check(&self, user_id: Uuid, content: &str, content_type: &str, score: f64, is_spam: bool) -> Result<(), SpamError> {
        sqlx::query(
            r#"
            INSERT INTO spam_checks (id, user_id, content_type, content_hash, score, is_spam, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            "#
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(content_type)
        .bind(self.hash_content(content))
        .bind(score)
        .bind(is_spam)
        .execute(&self.pool)
        .await
        .map_err(|e| SpamError::Database(e.to_string()))?;

        Ok(())
    }

    /// Логирование в audit_logs для отслеживания подозрительной активности
    async fn log_to_audit(&self, user_id: Uuid, content_type: &str, score: f64, reasons: &[String]) -> Result<(), SpamError> {
        sqlx::query(
            r#"
            INSERT INTO audit_logs (id, user_id, action, resource_type, resource_id, ip_address, user_agent, metadata, created_at)
            VALUES ($1, $2, $3, $4, NULL, '', '', $5, NOW())
            "#
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind("spam_check")
        .bind(content_type)
        .bind(serde_json::json!({
            "score": score,
            "reasons": reasons
        }))
        .execute(&self.pool)
        .await
        .map_err(|e| SpamError::Database(e.to_string()))?;

        Ok(())
    }
}

#[derive(Debug, serde::Serialize)]
pub struct SpamCheckResult {
    pub is_spam: bool,
    pub score: f64,
    pub action: SpamAction,
    pub reasons: Vec<String>,
}

#[derive(Debug, serde::Serialize, PartialEq)]
pub enum SpamAction {
    Allow,
    Moderate,
    Block,
}

#[derive(Debug)]
pub enum SpamError {
    Database(String),
}

impl std::fmt::Display for SpamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpamError::Database(e) => write!(f, "Database error: {}", e),
        }
    }
}

impl std::error::Error for SpamError {}

/// Миграция для spam_checks
pub fn create_spam_checks_table() -> &'static str {
    r#"
    CREATE TABLE IF NOT EXISTS spam_checks (
        id UUID PRIMARY KEY,
        user_id UUID NOT NULL REFERENCES users(id),
        content_type VARCHAR(50) NOT NULL,
        content_hash VARCHAR(64) NOT NULL,
        score DOUBLE PRECISION NOT NULL,
        is_spam BOOLEAN NOT NULL,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_spam_checks_user ON spam_checks(user_id);
    CREATE INDEX IF NOT EXISTS idx_spam_checks_created ON spam_checks(created_at DESC);
    CREATE INDEX IF NOT EXISTS idx_spam_checks_hash ON spam_checks(content_hash);
    "#
}
