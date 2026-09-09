use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures::future::{ok, LocalBoxFuture, Ready};
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;
use sysinfo::{CpuExt, System, SystemExt};
use tokio::sync::RwLock;

/// Глобальный счётчик активных подключений
static ACTIVE_CONNECTIONS: AtomicU32 = AtomicU32::new(0);

/// Увеличить счётчик подключений
pub fn inc_connections() {
    ACTIVE_CONNECTIONS.fetch_add(1, Ordering::SeqCst);
}

/// Уменьшить счётчик подключений
pub fn dec_connections() {
    ACTIVE_CONNECTIONS.fetch_sub(1, Ordering::SeqCst);
}

/// Получить текущее количество активных подключений
pub fn get_active_connections() -> u32 {
    ACTIVE_CONNECTIONS.load(Ordering::SeqCst)
}

#[derive(Debug, Clone, PartialEq)]
pub enum SystemLoad {
    Normal,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct SystemStatus {
    pub load: SystemLoad,
    pub active_connections: u32,
    pub cpu_usage: f32,
    pub memory_usage: f32,
}

pub struct DegradationConfig {
    pub high_load_threshold: u32,
    pub critical_load_threshold: u32,
    pub protected_paths: Vec<String>,
}

impl Default for DegradationConfig {
    fn default() -> Self {
        Self {
            high_load_threshold: 1000,
            critical_load_threshold: 2000,
            protected_paths: vec![
                "/health".to_string(),
                "/api/v1/auth".to_string(),
                "/api/v1/payments/webhook".to_string(), // Webhooks нельзя терять
                "/stream".to_string(),
            ],
        }
    }
}

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
            service: Rc::new(service),
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
    service: Rc<S>,
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
        let service = self.service.clone();

        Box::pin(async move {
            let current_status = status.read().await;

            // Проверяем, защищён ли путь
            let is_protected = config.protected_paths.iter().any(|p| path.starts_with(p));

            if is_protected {
                // Защищённые пути всегда пропускаем
                drop(current_status);
                let res = service.call(req).await?;
                return Ok(res);
            }

            match current_status.load {
                SystemLoad::Normal => {
                    drop(current_status);
                    let res = service.call(req).await?;
                    Ok(res)
                }
                SystemLoad::High => {
                    log::warn!("High system load, request to {} allowed", path);
                    drop(current_status);
                    let res = service.call(req).await?;
                    Ok(res)
                }
                SystemLoad::Critical => {
                    log::error!("Critical system load, rejecting request to {}", path);
                    drop(current_status);

                    Err(actix_web::error::ErrorServiceUnavailable(
                        "Service is under heavy load. Please try again later.",
                    ))
                }
            }
        })
    }
}

/// Монитор системных ресурсов с РЕАЛЬНЫМИ метриками
pub struct SystemMonitor {
    status: Arc<RwLock<SystemStatus>>,
    config: DegradationConfig,
    shutdown: tokio::sync::watch::Receiver<bool>,
}

impl SystemMonitor {
    pub fn new(config: DegradationConfig, shutdown: tokio::sync::watch::Receiver<bool>) -> Self {
        Self {
            status: Arc::new(RwLock::new(SystemStatus {
                load: SystemLoad::Normal,
                active_connections: 0,
                cpu_usage: 0.0,
                memory_usage: 0.0,
            })),
            config,
            shutdown,
        }
    }

    pub fn get_status(&self) -> Arc<RwLock<SystemStatus>> {
        self.status.clone()
    }

    /// Запуск мониторинга с graceful shutdown
    pub async fn run(&mut self) {
        log::info!("System monitor started");

        let mut sys = System::new_all();
        let mut refresh_counter = 0u32;

        loop {
            // Проверяем сигнал shutdown
            if *self.shutdown.borrow() {
                log::info!("System monitor shutting down");
                break;
            }

            // Обновляем CPU/RAM раз в 5 циклов (25 секунд) — это тяжёлая операция
            if refresh_counter % 5 == 0 {
                sys.refresh_all();
            }
            refresh_counter += 1;

            // РЕАЛЬНЫЕ метрики через sysinfo
            let cpu_usage = sys.global_cpu_info().cpu_usage();
            let memory_usage = {
                let total = sys.total_memory() as f32;
                let used = sys.used_memory() as f32;
                if total > 0.0 {
                    (used / total) * 100.0
                } else {
                    0.0
                }
            };
            let active_connections = ACTIVE_CONNECTIONS.load(Ordering::SeqCst);

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
            {
                let mut status = self.status.write().await;
                status.load = load.clone();
                status.active_connections = active_connections;
                status.cpu_usage = cpu_usage;
                status.memory_usage = memory_usage;
            }

            if load != SystemLoad::Normal {
                log::warn!(
                    "System load: {:?}, conn: {}, CPU: {:.1}%, RAM: {:.1}%",
                    load,
                    active_connections,
                    cpu_usage,
                    memory_usage
                );
            }

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
}

/// Middleware для подсчёта активных подключений
pub struct ConnectionCounter;

impl<S, B> Transform<S, ServiceRequest> for ConnectionCounter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = ConnectionCounterService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ConnectionCounterService {
            service: Rc::new(service),
        })
    }
}

pub struct ConnectionCounterService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for ConnectionCounterService<S>
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
        inc_connections();
        let service = self.service.clone();
        let fut = service.call(req);

        Box::pin(async move {
            let result = fut.await;
            dec_connections();
            result
        })
    }
}
