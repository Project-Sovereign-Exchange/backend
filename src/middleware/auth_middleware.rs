use crate::services::account::jwt_service::JwtService;
use actix_web::{
    Error, HttpMessage, HttpResponse,
    body::{BoxBody, MessageBody},
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode};
use serde::{Deserialize, Serialize};
use std::{
    future::{Ready, ready},
    rc::Rc,
};

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
                    let response = HttpResponse::Unauthorized().json(serde_json::json!({
                        "error": "Missing authentication token"
                    }));
                    return Ok(req.into_response(response));
                }
            };

            // Validate the JWT token
            let claims = match JwtService::validate_token(&token).await {
                Ok(claims) => claims,
                Err(err) => {
                    let response = HttpResponse::Unauthorized().json(serde_json::json!({
                        "error": format!("Invalid token: {}", err)
                    }));
                    return Ok(req.into_response(response));
                }
            };

            let path = req.path();

            match claims.purpose.as_str() {
                "temporary" => {
                    if !(path == "/mfa/verify") {
                        let response = HttpResponse::Forbidden()
                                .json(serde_json::json!({
                                    "error": "MFA verification required. This token can only access MFA verification endpoints."
                                }));
                        return Ok(req.into_response(response));
                    }
                }
                "access" => {
                    if path == "/mfa/verify" {
                        let response = HttpResponse::Forbidden().json(serde_json::json!({
                            "error": "Use MFA verification token for this endpoint"
                        }));
                        return Ok(req.into_response(response));
                    }
                }
                "admin" => {
                    if !path.starts_with("/admin") {
                        let response = HttpResponse::Forbidden().json(serde_json::json!({
                            "error": "Admin token can only access admin endpoints"
                        }));
                        return Ok(req.into_response(response));
                    }
                }
                _ => {
                    let response = HttpResponse::Unauthorized().json(serde_json::json!({
                        "error": "Invalid token purpose"
                    }));
                    return Ok(req.into_response(response));
                }
            }

            req.extensions_mut().insert(claims);

            let response = service.call(req).await?;
            Ok(response.map_into_boxed_body())
        })
    }
}

fn extract_jwt_from_cookie(req: &ServiceRequest) -> Option<String> {
    req.cookie("auth_token")
        .map(|cookie| cookie.value().to_string())
}
