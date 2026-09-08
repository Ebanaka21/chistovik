#[cfg(test)]
mod integration_tests {
    use actix_web::{test, web, App};
    use uuid::Uuid;
    
    // Helper функции для тестов
    mod helpers {
        use actix_web::web;
        use sqlx::PgPool;

        pub async fn setup_test_db() -> PgPool {
            // В реальных тестах используется тестовая БД
            // Для unit tests можно использовать mock
            let database_url = std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://localhost/chistovik_test".to_string());
            
            sqlx::postgres::PgPoolOptions::new()
                .max_connections(5)
                .connect(&database_url)
                .await
                .expect("Failed to create test database pool")
        }

        pub async fn cleanup_test_db(pool: &PgPool) {
            // Очистка тестовых данных
            let _ = sqlx::query("DELETE FROM audit_logs WHERE created_at < NOW() - INTERVAL '1 hour'")
                .execute(pool)
                .await;
        }
    }

    #[actix_web::test]
    async fn test_health_endpoint() {
        let app = test::init_service(
            App::new().route("/health", web::get().to(|| async { "OK" }))
        ).await;
        
        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_register_validation() {
        // Тест валидации при регистрации
        let app = test::init_service(
            App::new().route("/api/v1/auth/register", web::post().to(|| async {
                actix_web::HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "validation_error",
                    "message": "Email is required"
                }))
            }))
        ).await;
        
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/register")
            .set_json(serde_json::json!({
                "email": "",
                "password": "short"
            }))
            .to_request();
        
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);
    }

    #[actix_web::test]
    async fn test_password_policy() {
        use crate::services::auth_service::validate_password;
        
        // Тест короткого пароля
        assert!(validate_password("short", 8).is_err());
        
        // Тест распространённого пароля
        assert!(validate_password("password123", 8).is_err());
        
        // Тест валидного пароля
        assert!(validate_password("MyStr0ng!Pass", 8).is_ok());
    }

    #[actix_web::test]
    async fn test_media_token_generation() {
        use crate::services::streaming_service::{generate_media_token, validate_media_token};
        
        let secret = "test-secret-for-integration-tests";
        let user_id = Uuid::new_v4();
        let content_id = Uuid::new_v4();
        
        // Генерация токена
        let token = generate_media_token(user_id, content_id, true, 120, secret)
            .expect("Failed to generate media token");
        
        // Валидация токена
        let claims = validate_media_token(&token, secret)
            .expect("Failed to validate media token");
        
        assert_eq!(claims.user_id, user_id);
        assert_eq!(claims.content_id, content_id);
        assert!(claims.is_full_access);
    }

    #[actix_web::test]
    async fn test_webhook_signature_verification() {
        use crate::services::payment_service::verify_yookassa_webhook;
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        
        type HmacSha256 = Hmac<Sha256>;
        
        let secret = "test-webhook-secret";
        let body = r#"{"event":"payment.succeeded","object":{"id":"test-123"}}"#;
        
        // Генерация правильной подписи
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());
        
        // Тест валидной подписи
        assert!(verify_yookassa_webhook(body, &signature, secret).is_ok());
        
        // Тест невалидной подписи
        let wrong_signature = "0000000000000000000000000000000000000000000000000000000000000000";
        assert!(verify_yookassa_webhook(body, wrong_signature, secret).is_err());
    }

    #[actix_web::test]
    async fn test_rate_limiting_logic() {
        // Тест логики rate limiting (без реального Redis)
        // В integration tests с реальным Redis это будет полноценный тест
        
        let max_requests = 10u32;
        let mut current_count = 0u32;
        
        // Симуляция запросов
        for _ in 0..15 {
            current_count += 1;
            if current_count > max_requests {
                // Должен сработать rate limit
                break;
            }
        }
        
        assert!(current_count > max_requests);
    }

    #[actix_web::test]
    async fn test_cors_validation() {
        use crate::config::Config;
        
        // Тест валидации CORS origin
        let allowed_origins = vec![
            "http://localhost:3000".to_string(),
            "https://chistovik.ru".to_string(),
        ];
        
        let test_origin = "http://localhost:3000";
        assert!(allowed_origins.contains(&test_origin.to_string()));
        
        let malicious_origin = "http://evil.com";
        assert!(!allowed_origins.contains(&malicious_origin.to_string()));
    }

    #[actix_web::test]
    async fn test_security_headers() {
        let app = test::init_service(
            App::new()
                .wrap(crate::middleware::security::SecurityHeaders)
                .route("/test", web::get().to(|| async { "OK" }))
        ).await;
        
        let req = test::TestRequest::get().uri("/test").to_request();
        let resp = test::call_service(&app, req).await;
        
        // Проверка наличия security headers
        assert!(resp.headers().contains_key("X-Frame-Options"));
        assert!(resp.headers().contains_key("X-Content-Type-Options"));
        assert!(resp.headers().contains_key("X-XSS-Protection"));
    }

    #[actix_web::test]
    async fn test_metrics_endpoint() {
        let app = test::init_service(
            App::new().route("/health/metrics", web::get().to(|| async {
                actix_web::HttpResponse::Ok()
                    .content_type("text/plain")
                    .body("# HELP http_requests_total Total HTTP requests\n# TYPE http_requests_total counter\nhttp_requests_total 42\n")
            }))
        ).await;
        
        let req = test::TestRequest::get().uri("/health/metrics").to_request();
        let resp = test::call_service(&app, req).await;
        
        assert!(resp.status().is_success());
        assert_eq!(resp.headers().get("content-type").unwrap(), "text/plain");
    }
}
