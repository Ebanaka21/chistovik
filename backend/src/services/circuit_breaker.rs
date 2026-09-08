use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use std::future::Future;

/// Circuit Breaker — защита от каскадных отказов
/// Состояния: Closed (нормальная работа) → Open (блок) → HalfOpen (проверка)

#[derive(Debug, Clone, PartialEq)]
pub enum CircuitState {
    Closed,      // Всё работает
    Open,        // Блок запросов
    HalfOpen,    // Тестовый запрос
}

#[derive(Debug)]
struct CircuitBreakerInner {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure: Option<Instant>,
    opened_at: Option<Instant>,
}

/// Circuit Breaker конфигурация
pub struct CircuitBreaker {
    inner: Arc<RwLock<CircuitBreakerInner>>,
    failure_threshold: u32,      // Порог отказов для открытия
    success_threshold: u32,      // Порог успехов для закрытия
    timeout: Duration,           // Время в Open state
    name: String,
}

impl CircuitBreaker {
    pub fn new(name: &str, failure_threshold: u32, success_threshold: u32, timeout: Duration) -> Self {
        Self {
            inner: Arc::new(RwLock::new(CircuitBreakerInner {
                state: CircuitState::Closed,
                failure_count: 0,
                success_count: 0,
                last_failure: None,
                opened_at: None,
            })),
            failure_threshold,
            success_threshold,
            timeout,
            name: name.to_string(),
        }
    }

    /// Выполнение операции через circuit breaker
    pub async fn call<F, T, E>(&self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where
        F: FnOnce() -> std::pin::Pin<Box<dyn Future<Output = Result<T, E>> + Send>>,
    {
        let mut inner = self.inner.write().await;

        // Проверяем состояние
        match inner.state {
            CircuitState::Open => {
                // Проверяем, не истёк ли timeout
                if let Some(opened_at) = inner.opened_at {
                    if opened_at.elapsed() >= self.timeout {
                        // Переходим в HalfOpen
                        inner.state = CircuitState::HalfOpen;
                        inner.success_count = 0;
                        log::info!("Circuit breaker '{}' moved to HalfOpen", self.name);
                    } else {
                        return Err(CircuitBreakerError::CircuitOpen);
                    }
                } else {
                    return Err(CircuitBreakerError::CircuitOpen);
                }
            }
            CircuitState::HalfOpen => {
                // Разрешаем один тестовый запрос
            }
            CircuitState::Closed => {
                // Всё ок, пропускаем
            }
        }

        drop(inner);

        // Выполняем операцию
        let result = operation().await;

        let mut inner = self.inner.write().await;

        match result {
            Ok(value) => {
                inner.success_count += 1;
                
                if inner.state == CircuitState::HalfOpen && inner.success_count >= self.success_threshold {
                    inner.state = CircuitState::Closed;
                    inner.failure_count = 0;
                    inner.success_count = 0;
                    inner.opened_at = None;
                    log::info!("Circuit breaker '{}' closed", self.name);
                }

                Ok(value)
            }
            Err(error) => {
                inner.failure_count += 1;
                inner.last_failure = Some(Instant::now());

                if inner.state == CircuitState::HalfOpen || inner.failure_count >= self.failure_threshold {
                    inner.state = CircuitState::Open;
                    inner.opened_at = Some(Instant::now());
                    log::warn!("Circuit breaker '{}' opened after {} failures", self.name, inner.failure_count);
                }

                Err(CircuitBreakerError::OperationFailed(error))
            }
        }
    }

    /// Получение текущего состояния
    pub async fn state(&self) -> CircuitState {
        self.inner.read().await.state.clone()
    }

    /// Сброс circuit breaker
    pub async fn reset(&self) {
        let mut inner = self.inner.write().await;
        inner.state = CircuitState::Closed;
        inner.failure_count = 0;
        inner.success_count = 0;
        inner.last_failure = None;
        inner.opened_at = None;
    }
}

#[derive(Debug)]
pub enum CircuitBreakerError<E> {
    CircuitOpen,
    OperationFailed(E),
}

/// Retry с exponential backoff
pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
            jitter: true,
        }
    }
}

/// Выполнение операции с retry
pub async fn retry_with_backoff<F, Fut, T, E>(
    config: &RetryConfig,
    mut operation: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut delay = config.initial_delay;
    let mut attempt = 0;

    loop {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(error) => {
                attempt += 1;

                if attempt >= config.max_retries {
                    log::error!("Operation failed after {} attempts: {:?}", attempt, error);
                    return Err(error);
                }

                // Exponential backoff
                let mut sleep_duration = delay;

                if config.jitter {
                    // Добавляем случайность (±25%)
                    let jitter_range = sleep_duration.as_millis() as f64 * 0.25;
                    let jitter = (rand::random::<f64>() - 0.5) * 2.0 * jitter_range;
                    sleep_duration = Duration::from_millis((sleep_duration.as_millis() as f64 + jitter) as u64);
                }

                log::warn!(
                    "Operation failed (attempt {}/{}), retrying in {:?}: {:?}",
                    attempt,
                    config.max_retries,
                    sleep_duration,
                    error
                );

                tokio::time::sleep(sleep_duration).await;

                // Увеличиваем delay
                delay = Duration::from_millis((delay.as_millis() as f64 * config.multiplier) as u64);
                if delay > config.max_delay {
                    delay = config.max_delay;
                }
            }
        }
    }
}

/// Пример использования для ЮKassa
pub struct YookassaClient {
    client: reqwest::Client,
    circuit_breaker: CircuitBreaker,
    retry_config: RetryConfig,
    shop_id: String,
    secret_key: String,
}

impl YookassaClient {
    pub fn new(shop_id: &str, secret_key: &str) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap(),
            circuit_breaker: CircuitBreaker::new("yookassa", 5, 2, Duration::from_secs(60)),
            retry_config: RetryConfig {
                max_retries: 3,
                initial_delay: Duration::from_millis(200),
                max_delay: Duration::from_secs(10),
                multiplier: 2.0,
                jitter: true,
            },
            shop_id: shop_id.to_string(),
            secret_key: secret_key.to_string(),
        }
    }

    pub async fn create_payment(&self, body: serde_json::Value) -> Result<serde_json::Value, String> {
        let client = self.client.clone();
        let shop_id = self.shop_id.clone();
        let secret_key = self.secret_key.clone();

        retry_with_backoff(&self.retry_config, || {
            let client = client.clone();
            let shop_id = shop_id.clone();
            let secret_key = secret_key.clone();
            let body = body.clone();

            async move {
                let response = client
                    .post("https://api.yookassa.ru/v3/payments")
                    .basic_auth(&shop_id, Some(&secret_key))
                    .json(&body)
                    .send()
                    .await
                    .map_err(|e| format!("Request failed: {}", e))?;

                if response.status().is_success() {
                    response.json().await.map_err(|e| format!("Parse failed: {}", e))
                } else {
                    Err(format!("API error: {}", response.status()))
                }
            }
        }).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_circuit_breaker_opens_after_failures() {
        let cb = CircuitBreaker::new("test", 3, 2, Duration::from_secs(1));
        
        // 3 неудачи → должен открыться
        for _ in 0..3 {
            let _ = cb.call(|| Box::pin(async { Err::<(), _>("error") })).await;
        }

        assert_eq!(cb.state().await, CircuitState::Open);
    }

    #[tokio::test]
    async fn test_retry_with_backoff() {
        let config = RetryConfig {
            max_retries: 3,
            initial_delay: Duration::from_millis(10),
            max_delay: Duration::from_millis(100),
            multiplier: 2.0,
            jitter: false,
        };

        let mut attempt = 0;
        let result = retry_with_backoff(&config, || {
            attempt += 1;
            async move {
                if attempt < 3 {
                    Err("not ready")
                } else {
                    Ok("success")
                }
            }
        }).await;

        assert!(result.is_ok());
        assert_eq!(attempt, 3);
    }
}
