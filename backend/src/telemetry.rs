use opentelemetry::trace::TracerProvider;
use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{runtime, trace as sdktrace, Resource};
use prometheus::{Encoder, Registry, TextEncoder};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Глобальный registry для Prometheus метрик
lazy_static::lazy_static! {
    pub static ref METRICS_REGISTRY: Arc<Mutex<Registry>> = Arc::new(Mutex::new(Registry::new()));
}

/// Инициализация OpenTelemetry tracing
pub fn init_tracing(service_name: &str, otlp_endpoint: &str) -> Result<sdktrace::Tracer, Box<dyn std::error::Error>> {
    let resource = Resource::new(vec![
        KeyValue::new("service.name", service_name.to_string()),
        KeyValue::new("service.version", env!("CARGO_PKG_VERSION").to_string()),
    ]);

    let tracer_provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(otlp_endpoint),
        )
        .with_trace_config(sdktrace::config().with_resource(resource))
        .install_batch(runtime::Tokio)?;

    let tracer = tracer_provider.tracer(service_name);
    
    global::set_tracer_provider(tracer_provider);

    Ok(tracer)
}

/// Инициализация Prometheus метрик
pub fn init_metrics() -> Result<(), Box<dyn std::error::Error>> {
    // Метрики будут регистрироваться по мере использования
    Ok(())
}

/// Экспорт метрик в формате Prometheus
pub async fn export_metrics() -> String {
    let registry = METRICS_REGISTRY.lock().await;
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}

/// Метрики приложения
pub mod metrics {
    use prometheus::{Counter, Gauge, Histogram, HistogramOpts, IntCounter, IntGauge, Opts};
    use super::METRICS_REGISTRY;

    lazy_static::lazy_static! {
        /// Счётчик HTTP запросов
        pub static ref HTTP_REQUESTS_TOTAL: IntCounter = IntCounter::with_opts(
            Opts::new("http_requests_total", "Total number of HTTP requests")
        ).unwrap();

        /// Счётчик HTTP ошибок
        pub static ref HTTP_ERRORS_TOTAL: IntCounter = IntCounter::with_opts(
            Opts::new("http_errors_total", "Total number of HTTP errors")
        ).unwrap();

        /// Гистограмма длительности HTTP запросов
        pub static ref HTTP_REQUEST_DURATION: Histogram = Histogram::with_opts(
            HistogramOpts::new("http_request_duration_seconds", "HTTP request duration in seconds")
                .buckets(vec![0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0])
        ).unwrap();

        /// Количество активных подключений к БД
        pub static ref DB_CONNECTIONS_ACTIVE: IntGauge = IntGauge::with_opts(
            Opts::new("db_connections_active", "Number of active database connections")
        ).unwrap();

        /// Счётчик успешных аутентификаций
        pub static ref AUTH_SUCCESS_TOTAL: IntCounter = IntCounter::with_opts(
            Opts::new("auth_success_total", "Total number of successful authentications")
        ).unwrap();

        /// Счётчик неудачных аутентификаций
        pub static ref AUTH_FAILURE_TOTAL: IntCounter = IntCounter::with_opts(
            Opts::new("auth_failure_total", "Total number of failed authentications")
        ).unwrap();

        /// Счётчик созданных подписок
        pub static ref SUBSCRIPTIONS_CREATED_TOTAL: IntCounter = IntCounter::with_opts(
            Opts::new("subscriptions_created_total", "Total number of subscriptions created")
        ).unwrap();

        /// Счётчик успешных платежей
        pub static ref PAYMENTS_SUCCESS_TOTAL: IntCounter = IntCounter::with_opts(
            Opts::new("payments_success_total", "Total number of successful payments")
        ).unwrap();

        /// Счётчик стриминг запросов
        pub static ref STREAMING_REQUESTS_TOTAL: IntCounter = IntCounter::with_opts(
            Opts::new("streaming_requests_total", "Total number of streaming requests")
        ).unwrap();

        /// Гистограмма размера загружаемых файлов
        pub static ref UPLOAD_SIZE_BYTES: Histogram = Histogram::with_opts(
            HistogramOpts::new("upload_size_bytes", "Size of uploaded files in bytes")
                .buckets(vec![1024.0, 10240.0, 102400.0, 1048576.0, 10485760.0, 104857600.0])
        ).unwrap();
    }

    /// Регистрация всех метрик в Prometheus registry
    pub async fn register_all() {
        let mut registry = METRICS_REGISTRY.lock().await;
        
        let _ = registry.register(Box::new(HTTP_REQUESTS_TOTAL.clone()));
        let _ = registry.register(Box::new(HTTP_ERRORS_TOTAL.clone()));
        let _ = registry.register(Box::new(HTTP_REQUEST_DURATION.clone()));
        let _ = registry.register(Box::new(DB_CONNECTIONS_ACTIVE.clone()));
        let _ = registry.register(Box::new(AUTH_SUCCESS_TOTAL.clone()));
        let _ = registry.register(Box::new(AUTH_FAILURE_TOTAL.clone()));
        let _ = registry.register(Box::new(SUBSCRIPTIONS_CREATED_TOTAL.clone()));
        let _ = registry.register(Box::new(PAYMENTS_SUCCESS_TOTAL.clone()));
        let _ = registry.register(Box::new(STREAMING_REQUESTS_TOTAL.clone()));
        let _ = registry.register(Box::new(UPLOAD_SIZE_BYTES.clone()));
    }
}

/// Middleware для записи метрик HTTP запросов
pub mod middleware {
    use actix_web::{dev::{ServiceRequest, ServiceResponse, Transform, Service}, Error, http::StatusCode};
    use futures::future::{ok, Ready, LocalBoxFuture};
    use std::task::{Context, Poll};
    use std::time::Instant;

    use super::metrics::{HTTP_REQUESTS_TOTAL, HTTP_ERRORS_TOTAL, HTTP_REQUEST_DURATION};

    pub struct MetricsMiddleware;

    impl<S, B> Transform<S, ServiceRequest> for MetricsMiddleware
    where
        S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
        B: 'static,
    {
        type Response = ServiceResponse<B>;
        type Error = Error;
        type Transform = MetricsMiddlewareService<S>;
        type InitError = ();
        type Future = Ready<Result<Self::Transform, Self::InitError>>;

        fn new_transform(&self, service: S) -> Self::Future {
            ok(MetricsMiddlewareService { service })
        }
    }

    pub struct MetricsMiddlewareService<S> {
        service: S,
    }

    impl<S, B> Service<ServiceRequest> for MetricsMiddlewareService<S>
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
            let start = Instant::now();
            let method = req.method().to_string();
            let path = req.path().to_string();

            let fut = self.service.call(req);

            Box::pin(async move {
                let res = fut.await?;
                let duration = start.elapsed().as_secs_f64();
                let status = res.status();

                // Инкремент счётчика запросов
                HTTP_REQUESTS_TOTAL.inc();

                // Инкремент счётчика ошибок (5xx)
                if status.is_server_error() {
                    HTTP_ERRORS_TOTAL.inc();
                }

                // Запись длительности запроса
                HTTP_REQUEST_DURATION.observe(duration);

                // Логирование медленных запросов
                if duration > 1.0 {
                    log::warn!(
                        "Slow request: {} {} - {}ms",
                        method,
                        path,
                        (duration * 1000.0) as u64
                    );
                }

                Ok(res)
            })
        }
    }
}
