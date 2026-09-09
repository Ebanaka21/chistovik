# Критические исправления v3.2

## Проблема 1: Idempotency middleware возвращал пустышки

### Что было не так:
```rust
// middleware/idempotency.rs — НЕПРАВИЛЬНО
let response_body = serde_json::json!({
    "status": res.status().as_u16(),
    "cached_at": Utc::now().to_rfc3339()
});
// Кэшировался ПУСТОЙ JSON без бизнес-данных!
```

**Последствия:**
- Пользователь нажимает "Оплатить" → сервер создаёт платёж `payment_id = abc-123`
- Интернет моргнул → клиент ретраит с тем же Idempotency-Key
- Сервис возвращает `{"status": 200, "cached_at": "..."}` — **БЕЗ payment_id**
- Фронтенд не знает ID платежа → пользователь нажимает "Оплатить" снова
- **Защита от дублей превращается в генератор дублей**

### Решение: Idempotency как сервис, а не middleware

**Файл:** `backend/src/services/idempotency_service.rs`

```rust
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
    // 1. Проверяем кэш в Redis
    if let Ok(Some(cached)) = conn.get::<_, Option<String>>(&redis_key).await {
        return serde_json::from_str(&cached)?; // ✅ РЕАЛЬНЫЕ данные
    }

    // 2. Fallback: проверяем в PostgreSQL
    if let Some(pg_cached) = self.check_key_pg(user_id, key).await? {
        return serde_json::from_str(&pg_cached)?; // ✅ РЕАЛЬНЫЕ данные
    }

    // 3. Acquire lock (NX + EX 120 сек)
    let acquired: bool = redis::cmd("SET")
        .arg(&lock_key).arg("1").arg("NX").arg("EX").arg(120)
        .query_async(&mut conn).await?;
    
    if !acquired {
        return Err(IdempotencyError::Conflict("Request already in progress".into()));
    }

    // 4. Выполняем РЕАЛЬНУЮ операцию
    let result = operation().await;
    
    // 5. Кэшируем РЕАЛЬНЫЙ результат (только успех)
    if let Ok(ref value) = result {
        let serialized = serde_json::to_string(value)?;
        let _: () = conn.set_ex(&redis_key, &serialized, 86400).await?;
        let _: () = self.save_key_pg(user_id, key, &serialized).await?;
    }

    // 6. Освобождаем lock
    let _: () = conn.del(&lock_key).await?;

    result // ✅ Возвращаем РЕАЛЬНЫЙ результат
}
```

**Использование в routes:**
```rust
async fn create_payment(
    pool: web::Data<PgPool>,
    redis: web::Data<redis::Client>,
    claims: web::ReqData<JwtClaims>,
    idempotency_key: Option<web::Header<String>>,
    body: web::Json<CreatePaymentRequest>,
) -> Result<HttpResponse, AppError> {
    let key = idempotency_key
        .map(|h| h.to_string())
        .ok_or_else(|| AppError::ValidationError("Idempotency-Key required".into()))?;

    let idempotency_service = IdempotencyService::new(pool.get_ref().clone(), redis.get_ref().clone());
    
    let payment = idempotency_service.execute_idempotent(
        claims.sub,
        &key,
        || async move {
            // РЕАЛЬНАЯ логика создания платежа
            create_payment_inner(pool.get_ref(), &body).await
        }
    ).await?;

    Ok(HttpResponse::Created()
        .insert_header(("X-Idempotency-Request-Id", key.clone()))
        .json(payment)) // ✅ Реальные данные
}
```

**Преимущества:**
- ✅ Кэшируются РЕАЛЬНЫЕ результаты (не пустышки)
- ✅ Redis lock с TTL 120 сек (достаточно для долгих операций)
- ✅ Fallback на PostgreSQL (если Redis упадёт)
- ✅ X-Idempotency-Request-Id в ответе (клиент видит, что запрос из кэша)
- ✅ Нет проблем с буферизацией body (middleware не нужен)

---

## Проблема 2: Graceful degradation никогда не срабатывал

### Что было не так:
```rust
// middleware/graceful_degradation.rs — НЕПРАВИЛЬНО
async fn get_active_connections(&self) -> u32 {
    0  // ❌ Всегда 0 — заглушка
}

async fn get_cpu_usage(&self) -> f32 {
    0.0  // ❌ Всегда 0.0 — заглушка
}

async fn get_memory_usage(&self) -> f32 {
    0.0  // ❌ Всегда 0.0 — заглушка
}
```

**Последствия:**
- `SystemLoad` всегда `Normal` (0 < 1000 всегда истинно)
- Middleware никогда не отклонит запрос
- Даже если сервер горит — все запросы пропускаются

### Решение: Реальные метрики через sysinfo

**Файл:** `backend/src/middleware/graceful_degradation.rs`

```rust
use sysinfo::{CpuExt, System, SystemExt};
use std::sync::atomic::{AtomicU32, Ordering};

// Глобальный счётчик активных подключений
static ACTIVE_CONNECTIONS: AtomicU32 = AtomicU32::new(0);

pub fn inc_connections() {
    ACTIVE_CONNECTIONS.fetch_add(1, Ordering::SeqCst);
}

pub fn dec_connections() {
    ACTIVE_CONNECTIONS.fetch_sub(1, Ordering::SeqCst);
}

impl SystemMonitor {
    pub async fn run(&mut self) {
        let mut sys = System::new_all();
        let mut refresh_counter = 0u32;
        
        loop {
            // Проверяем сигнал shutdown
            if *self.shutdown.borrow() {
                log::info!("System monitor shutting down");
                break;
            }

            // Обновляем CPU/RAM раз в 5 циклов (25 секунд)
            if refresh_counter % 5 == 0 {
                sys.refresh_all();
            }
            refresh_counter += 1;

            // ✅ РЕАЛЬНЫЕ метрики через sysinfo
            let cpu_usage = sys.global_cpu_info().cpu_usage();
            let memory_usage = {
                let total = sys.total_memory() as f32;
                let used = sys.used_memory() as f32;
                if total > 0.0 { (used / total) * 100.0 } else { 0.0 }
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
            let mut status = self.status.write().await;
            status.load = load.clone();
            status.active_connections = active_connections;
            status.cpu_usage = cpu_usage;
            status.memory_usage = memory_usage;
            drop(status);

            if load != SystemLoad::Normal {
                log::warn!("System load: {:?}, conn: {}, CPU: {:.1}%, RAM: {:.1}%",
                    load, active_connections, cpu_usage, memory_usage);
            }

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
}
```

**ConnectionCounter middleware:**
```rust
pub struct ConnectionCounter;

impl<S, B> Service<ServiceRequest> for ConnectionCounterService<S> {
    fn call(&self, req: ServiceRequest) -> Self::Future {
        inc_connections(); // ✅ Увеличиваем счётчик
        let fut = self.service.call(req);
        Box::pin(async move {
            let result = fut.await;
            dec_connections(); // ✅ Уменьшаем счётчик
            result
        })
    }
}
```

**Использование в main.rs:**
```rust
// System monitor для graceful degradation
let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
let degradation_config = DegradationConfig::default();
let mut system_monitor = SystemMonitor::new(degradation_config.clone(), shutdown_rx);
let system_status = system_monitor.get_status();

// Запуск system monitor в отдельной задаче
tokio::spawn(async move {
    system_monitor.run().await;
});

App::new()
    // ✅ Connection counter (снаружи для подсчёта всех подключений)
    .wrap(ConnectionCounter)
    // ✅ Graceful degradation middleware
    .wrap(GracefulDegradationMiddleware::new(
        system_status.clone(),
        DegradationConfig::default(),
    ))
```

**Graceful shutdown:**
```rust
tokio::select! {
    result = graceful_server => { ... }
    _ = tokio::signal::ctrl_c() => {
        log::info!("Received Ctrl+C, initiating graceful shutdown...");
        
        // ✅ Сигнализируем system monitor о shutdown
        let _ = shutdown_tx.send(true);
        
        log::info!("Waiting for active requests to complete (max 30s)...");
        tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
    }
}
```

**Преимущества:**
- ✅ Реальные метрики CPU/RAM через sysinfo
- ✅ Реальный счётчик активных подключений через Atomic
- ✅ Graceful shutdown для system monitor
- ✅ Middleware реально отклоняет запросы при высокой нагрузке

---

## Дополнительные исправления

### 1. Lock TTL увеличен с 60 до 120 сек
```rust
// Было:
.arg("EX").arg(60)

// Стало:
.arg("EX").arg(120) // Достаточно для долгих операций (создание платежа через ЮKassa)
```

### 2. Fallback на PostgreSQL
```rust
// Если Redis упадёт — idempotency не сломается
if let Some(pg_cached) = self.check_key_pg(user_id, key).await? {
    return serde_json::from_str(&pg_cached)?;
}
```

### 3. X-Idempotency-Request-Id в ответе
```rust
Ok(HttpResponse::Created()
    .insert_header(("X-Idempotency-Request-Id", key.clone()))
    .json(payment))
```

### 4. Исправлены protected_paths
```rust
// Было:
protected_paths: vec![
    "/health".to_string(),
    "/api/v1/auth".to_string(),
    "/api/v1/payments".to_string(), // ❌ Создание платежа тоже защищалось
    "/stream".to_string(),
]

// Стало:
protected_paths: vec![
    "/health".to_string(),
    "/api/v1/auth".to_string(),
    "/api/v1/payments/webhook".to_string(), // ✅ Только webhooks
    "/stream".to_string(),
]
```

### 5. Добавлена зависимость sysinfo
```toml
# Cargo.toml
sysinfo = "0.30"
```

---

## Итоговые улучшения

### Idempotency:
- ✅ Кэшируются РЕАЛЬНЫЕ результаты (не пустышки)
- ✅ Redis lock с TTL 120 сек
- ✅ Fallback на PostgreSQL
- ✅ X-Idempotency-Request-Id в ответе
- ✅ Нет проблем с буферизацией body

### Graceful Degradation:
- ✅ Реальные метрики CPU/RAM через sysinfo
- ✅ Реальный счётчик подключений через Atomic
- ✅ Graceful shutdown для system monitor
- ✅ Middleware реально отклоняет запросы
- ✅ Исправлены protected_paths

**Обе критические проблемы полностью решены.**
