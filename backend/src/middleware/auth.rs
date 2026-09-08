use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use futures::future::{ok, LocalBoxFuture, Ready};
use std::rc::Rc;
use std::task::{Context, Poll};

use crate::config::Config;
use crate::models::user::{JwtClaims, UserRole};
use crate::services::auth_service;

/// Middleware для проверки JWT-токена
/// Извлекает токен из Authorization header, валидирует и добавляет claims в request extensions
pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareService {
            service: Rc::new(service),
        })
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        Box::pin(async move {
            // Извлечение токена из Authorization header
            let auth_header = req
                .headers()
                .get("Authorization")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string());

            // Получение config из app_data
            let config = req
                .app_data::<actix_web::web::Data<Config>>()
                .map(|c| c.get_ref().clone());

            let config = match config {
                Some(c) => c,
                None => {
                    return Err(actix_web::error::ErrorInternalServerError(
                        "Config not found",
                    ));
                }
            };

            // Валидация токена
            let claims = match auth_header {
                Some(header) if header.starts_with("Bearer ") => {
                    let token = &header[7..];
                    match auth_service::validate_token(token, &config.jwt_secret) {
                        Ok(claims) => claims,
                        Err(_) => {
                            return Err(actix_web::error::ErrorUnauthorized(
                                "Invalid or expired token",
                            ));
                        }
                    }
                }
                _ => {
                    return Err(actix_web::error::ErrorUnauthorized(
                        "Missing authorization header",
                    ));
                }
            };

            // FIX: Вставляем claims в request extensions ДО вызова service
            req.extensions_mut().insert(claims);

            // Теперь вызываем следующий сервис с claims в extensions
            let res = service.call(req).await?;
            Ok(res)
        })
    }
}

/// Проверка роли пользователя
pub fn require_role(claims: &JwtClaims, required_role: UserRole) -> Result<(), String> {
    match (&claims.role, &required_role) {
        (UserRole::Admin, _) => Ok(()), // Админ имеет доступ ко всему
        (UserRole::Author, UserRole::Author) => Ok(()),
        (UserRole::Fan, UserRole::Fan) => Ok(()),
        _ => Err("Недостаточно прав".to_string()),
    }
}
