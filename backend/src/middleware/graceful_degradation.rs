use actix_web::{dev::{ServiceRequest, ServiceResponse, Transform, Service}, Error, HttpResponse, web};
use futures::future::{ok, Ready, LocalBoxFuture};
use std::task::{Context, Poll};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Graceful Degradation Middleware
/// При высокой нагрузке отклоняет некритичные запросы, сохраняя работоспособность критичных

#[derive(Debug, Clone, PartialEq)]
pub enum SystemLoad {
    Normal,
    High,
    Critical,
}

/// Статус системы
#[derive(Debug, Clone)]
pub struct SystemStatus {
    pub load: SystemLoad,
    pub active_connections: u32,
    pub cpu_usage: f32,
    pub memory_usage: f32,
}

/// Конфигурация graceful degradation
pub struct DegradationConfig {
    pub high_load_threshold: u32,      // Порог высокой нагрузки
    pub critical_load_threshold: u32,  // Порог критической нагрузки
    pub protected_paths: Vec<String>,  // Пути, которые не блокируются
}

impl Default for DegradationConfig {
    fn default() -> Self {
        Self {
            high_load_threshold: 1000,
            critical_load_threshold: 2000,
            protected_paths: vec![
                "/health".to_string(),
                "/api/v1/auth".to_string(),
                "/api/v1/payments".to_string(),
                "/stream".to_string(),
            ],
        }
    }
}

/// Middleware для graceful degradation
pub struct GracefulDegradationMiddleware {
    status: Arc<RwLock<SystemStatus>>,
    config: DegradationConfig,
}

impl GracefulDegradationMiddleware {
    pub fn new(status: Arc<RwLock<SystemStatus>>, config: DegradationConfig) -> Self {
        Self { status, config }
    }
}

impl<S, B> Transform<S, ServiceRequest> for GracefulDegradationMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = GracefulDegradationService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(GracefulDegradationService {
            service,
            status: self.status.clone(),
            config: DegradationConfig {
                high_load_threshold: self.config.high_load_threshold,
                critical_load_threshold: self.config.critical_load_threshold,
                protected_paths: self.config.protected_paths.clone(),
            },
        })
    }
}

pub struct GracefulDegradationService<S> {
    service: S,
    status: Arc<RwLock<SystemStatus>>,
    config: DegradationConfig,
}

impl<S, B> Service<ServiceRequest> for GracefulDegradationService<S>
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
        let path = req.path().to_string();
        let status = self.status.clone();
        let config = self.config.clone();
        let fut = self.service.call(req);

        Box::pin(async move {
            let current_status = status.read().await;
            
            // Проверяем, защищён ли путь
            let is_protected = config.protected_paths.iter().any(|p| path.starts_with(p));
            
            if is_protected {
                // Защищённые пути всегда пропускаем
                return fut.await;
            }

            match current_status.load {
                SystemLoad::Normal => {
                    // Нормальная нагрузка — пропускаем
                    fut.await
                }
                SystemLoad::High => {
                    // Высокая нагрузка — пропускаем, но логируем
                    log::warn!("High system load, request to {} allowed", path);
                    fut.await
                }
                SystemLoad::Critical => {
                    // Критическая нагрузка — отклоняем некритичные запросы
                    log::error!("Critical system load, rejecting request to {}", path);
                    
                    Err(actix_web::error::ErrorServiceUnavailable(
                        "Service is under heavy load. Please try again later."
                    ))
                }
            }
        })
    }
}

/// Монитор системных ресурсов
pub struct SystemMonitor {
    status: Arc<RwLock<SystemStatus>>,
    config: DegradationConfig,
}

impl SystemMonitor {
    pub fn new(config: DegradationConfig) -> Self {
        Self {
            status: Arc::new(RwLock::new(SystemStatus {
                load: SystemLoad::Normal,
                active_connections: 0,
                cpu_usage: 0.0,
                memory_usage: 0.0,
            })),
            config,
        }
    }

    pub fn get_status(&self) -> Arc<RwLock<SystemStatus>> {
        self.status.clone()
    }

    /// Запуск мониторинга
    pub async fn run(&self) {
        log::info!("System monitor started");

        loop {
            // Получаем метрики системы
            let active_connections = self.get_active_connections().await;
            let cpu_usage = self.get_cpu_usage().await;
            let memory_usage = self.get_memory_usage().await;

            // Определяем уровень нагрузки
            let load = if active_connections > self.config.critical_load_threshold 
                || cpu_usage > 90.0 
                || memory_usage > 90.0 
            {
                SystemLoad::Critical
            } else if active_connections > self.config.high_load_threshold 
                || cpu_usage > 70.0 
                || memory_usage > 70.0 
            {
                SystemLoad::High
            } else {
                SystemLoad::Normal
            };

            // Обновляем статус
            let mut status = self.status.write().await;
            status.load = load;
            status.active_connections = active_connections;
            status.cpu_usage = cpu_usage;
            status.memory_usage = memory_usage;

            if load != SystemLoad::Normal {
                log::warn!("System load: {:?}, connections: {}, CPU: {:.1}%, Memory: {:.1}%", 
                    load, active_connections, cpu_usage, memory_usage);
            }

            // Проверяем каждые 5 секунд
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }

    /// Получение количества активных подключений
    async fn get_active_connections(&self) -> u32 {
        // В реальности — получение из actix-web server stats
        // Пока — заглушка
        0
    }

    /// Получение CPU usage
    async fn get_cpu_usage(&self) -> f32 {
        // В реальности — чтение из /proc/stat или sysinfo crate
        // Пока — заглушка
        0.0
    }

    /// Получение memory usage
    async fn get_memory_usage(&self) -> f32 {
        // В реальности — чтение из /proc/meminfo или sysinfo crate
        // Пока — заглушка
        0.0
    }
}
