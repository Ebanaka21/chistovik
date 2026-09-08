use sqlx::PgPool;
use uuid::Uuid;
use chrono::{Utc, Duration};
use serde::Serialize;

/// Сервис обнаружения подозрительной активности
pub struct AnomalyDetectionService {
    pool: PgPool,
}

impl AnomalyDetectionService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Проверка подозрительной активности при входе
    pub async fn check_login_anomaly(&self, user_id: Uuid, ip_address: &str, user_agent: &str) -> Result<AnomalyResult, AnomalyError> {
        let mut alerts = Vec::new();
        let mut risk_score = 0.0;

        // 1. Проверка нового IP
        let ip_usage_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(DISTINCT user_id) FROM user_sessions WHERE ip_address = $1 AND created_at > NOW() - INTERVAL '7 days'"
        )
        .bind(ip_address)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AnomalyError::Database(e.to_string()))?;

        if ip_usage_count > 5 {
            risk_score += 0.3;
            alerts.push(format!("IP used by {} different users", ip_usage_count));
        }

        // 2. Проверка географической аномалии (если IP из другой страны)
        // В реальности — интеграция с GeoIP сервисом
        
        // 3. Проверка скорости входов
        let recent_logins: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM audit_logs WHERE user_id = $1 AND action = 'login' AND created_at > NOW() - INTERVAL '1 hour'"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AnomalyError::Database(e.to_string()))?;

        if recent_logins > 10 {
            risk_score += 0.4;
            alerts.push("Too many login attempts in 1 hour".to_string());
        }

        // 4. Проверка смены устройства
        let device_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(DISTINCT user_agent) FROM user_sessions WHERE user_id = $1 AND created_at > NOW() - INTERVAL '24 hours'"
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AnomalyError::Database(e.to_string()))?;

        if device_count > 3 {
            risk_score += 0.2;
            alerts.push(format!("{} different devices in 24 hours", device_count));
        }

        // 5. Проверка времени входа (ночные часы)
        let hour = Utc::now().hour();
        if hour >= 2 && hour <= 5 {
            risk_score += 0.1;
            alerts.push("Login during unusual hours (2-5 AM)".to_string());
        }

        let is_suspicious = risk_score >= 0.6;
        let severity = if risk_score >= 0.8 {
            Severity::High
        } else if risk_score >= 0.6 {
            Severity::Medium
        } else {
            Severity::Low
        };

        // Логируем алерт
        if is_suspicious {
            self.log_alert(user_id, "suspicious_login", risk_score, &alerts, ip_address).await?;
        }

        Ok(AnomalyResult {
            is_suspicious,
            risk_score,
            severity,
            alerts,
        })
    }

    /// Проверка подозрительной активности при действиях
    pub async fn check_action_anomaly(&self, user_id: Uuid, action: &str, metadata: &serde_json::Value) -> Result<AnomalyResult, AnomalyError> {
        let mut alerts = Vec::new();
        let mut risk_score = 0.0;

        match action {
            "payment" => {
                // Проверка необычно большой суммы
                if let Some(amount) = metadata.get("amount_kopecks").and_then(|v| v.as_i64()) {
                    let avg_amount: i64 = sqlx::query_scalar(
                        "SELECT COALESCE(AVG(amount_kopecks), 0)::BIGINT FROM payments WHERE user_id = $1 AND created_at > NOW() - INTERVAL '30 days'"
                    )
                    .bind(user_id)
                    .fetch_one(&self.pool)
                    .await
                    .map_err(|e| AnomalyError::Database(e.to_string()))?;

                    if amount > avg_amount * 3 && avg_amount > 0 {
                        risk_score += 0.5;
                        alerts.push(format!("Payment amount {} is 3x higher than average {}", amount, avg_amount));
                    }
                }
            }
            "content_upload" => {
                // Проверка необычно частых загрузок
                let recent_uploads: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM audit_logs WHERE user_id = $1 AND action = 'content_upload' AND created_at > NOW() - INTERVAL '1 hour'"
                )
                .bind(user_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| AnomalyError::Database(e.to_string()))?;

                if recent_uploads > 10 {
                    risk_score += 0.4;
                    alerts.push(format!("{} uploads in 1 hour", recent_uploads));
                }
            }
            "subscription_cancel" => {
                // Проверка массовых отмен
                let recent_cancels: i64 = sqlx::query_scalar(
                    "SELECT COUNT(*) FROM audit_logs WHERE user_id = $1 AND action = 'subscription_cancel' AND created_at > NOW() - INTERVAL '24 hours'"
                )
                .bind(user_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|e| AnomalyError::Database(e.to_string()))?;

                if recent_cancels > 5 {
                    risk_score += 0.3;
                    alerts.push(format!("{} subscription cancellations in 24 hours", recent_cancels));
                }
            }
            _ => {}
        }

        let is_suspicious = risk_score >= 0.6;
        let severity = if risk_score >= 0.8 {
            Severity::High
        } else if risk_score >= 0.6 {
            Severity::Medium
        } else {
            Severity::Low
        };

        if is_suspicious {
            self.log_alert(user_id, action, risk_score, &alerts, "").await?;
        }

        Ok(AnomalyResult {
            is_suspicious,
            risk_score,
            severity,
            alerts,
        })
    }

    /// Логирование алерта
    async fn log_alert(&self, user_id: Uuid, alert_type: &str, risk_score: f64, alerts: &[String], ip_address: &str) -> Result<(), AnomalyError> {
        sqlx::query(
            r#"
            INSERT INTO security_alerts (id, user_id, alert_type, risk_score, alerts, ip_address, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, NOW())
            "#
        )
        .bind(Uuid::new_v4())
        .bind(user_id)
        .bind(alert_type)
        .bind(risk_score)
        .bind(serde_json::to_value(alerts).unwrap_or_default())
        .bind(ip_address)
        .execute(&self.pool)
        .await
        .map_err(|e| AnomalyError::Database(e.to_string()))?;

        // Отправка уведомления администратору (если высокий риск)
        if risk_score >= 0.8 {
            // В реальности — отправка email/Telegram уведомления
            log::warn!("High-risk security alert for user {}: {:?}", user_id, alerts);
        }

        Ok(())
    }

    /// Получение недавних алертов
    pub async fn get_recent_alerts(&self, limit: i64) -> Result<Vec<SecurityAlert>, AnomalyError> {
        let alerts = sqlx::query_as::<_, SecurityAlert>(
            r#"
            SELECT id, user_id, alert_type, risk_score, alerts, ip_address, created_at
            FROM security_alerts
            ORDER BY created_at DESC
            LIMIT $1
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AnomalyError::Database(e.to_string()))?;

        Ok(alerts)
    }
}

#[derive(Debug, Serialize)]
pub struct AnomalyResult {
    pub is_suspicious: bool,
    pub risk_score: f64,
    pub severity: Severity,
    pub alerts: Vec<String>,
}

#[derive(Debug, Serialize)]
pub enum Severity {
    Low,
    Medium,
    High,
}

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct SecurityAlert {
    pub id: Uuid,
    pub user_id: Uuid,
    pub alert_type: String,
    pub risk_score: f64,
    pub alerts: serde_json::Value,
    pub ip_address: String,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug)]
pub enum AnomalyError {
    Database(String),
}

impl std::fmt::Display for AnomalyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AnomalyError::Database(e) => write!(f, "Database error: {}", e),
        }
    }
}

impl std::error::Error for AnomalyError {}

/// Миграция для security_alerts
pub fn create_security_alerts_table() -> &'static str {
    r#"
    CREATE TABLE IF NOT EXISTS security_alerts (
        id UUID PRIMARY KEY,
        user_id UUID NOT NULL REFERENCES users(id),
        alert_type VARCHAR(100) NOT NULL,
        risk_score DOUBLE PRECISION NOT NULL,
        alerts JSONB NOT NULL DEFAULT '[]',
        ip_address VARCHAR(45),
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_security_alerts_user ON security_alerts(user_id);
    CREATE INDEX IF NOT EXISTS idx_security_alerts_created ON security_alerts(created_at DESC);
    CREATE INDEX IF NOT EXISTS idx_security_alerts_type ON security_alerts(alert_type);
    "#
}
