use actix_web::{
    body::{BoxBody, MessageBody},
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::{
    future::{ready, Ready},
    rc::Rc,
};
use crate::services::jwt_service::JwtService;

pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);

        Box::pin(async move {
            // Extract JWT token from HTTP-only cookie
            let token = match extract_jwt_from_cookie(&req) {
                Some(token) => token,
                None => {
                    let response = HttpResponse::Unauthorized()
                        .json(serde_json::json!({
                            "error": "Missing authentication token"
                        }));
                    return Ok(req.into_response(response));
                }
            };

            // Validate the JWT token
            let claims = match JwtService::validate_token(&token).await {
                Ok(claims) => claims,
                Err(err) => {
                    let response = HttpResponse::Unauthorized()
                        .json(serde_json::json!({
                            "error": format!("Invalid token: {}", err)
                        }));
                    return Ok(req.into_response(response));
                }
            };

            // Add claims to request extensions for use in handlers
            req.extensions_mut().insert(claims);

            // Continue to the next service
            let response = service.call(req).await?;
            Ok(response.map_into_boxed_body())
        })
    }
}

fn extract_jwt_from_cookie(req: &ServiceRequest) -> Option<String> {
    req.cookie("auth_token")
        .map(|cookie| cookie.value().to_string())
}