use actix_web::HttpRequest;
use actix_web::{get, post, web, HttpResponse, Responder, Result};
use actix_web::cookie::{Cookie, SameSite};
use actix_web::http::header;
use futures_util::TryFutureExt;
use serde::{Deserialize, Serialize};
use tracing::log;
use crate::app_state::AppState;
use crate::services::account::auth_service::{AuthService, AuthenticatedUser};
use crate::services::account::jwt_service::{Claims, JwtService};
use crate::services::account::user_service::UserService;
use crate::services::integrations::cookie_service::CookieService;
use crate::utils::validator_util::ValidatorUtil;

//Login Route
#[derive(Deserialize)]
struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct MeResponse {
    pub user: UserResponse,
    pub token_purpose: String,
    pub issued_at: i64,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub username: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

#[post("/login")]
async fn login(
    state: web::Data<AppState>,
    request: web::Json<LoginRequest>,
) -> Result<impl Responder> {
    let auth_service = AuthService::new(state.as_ref().clone());
    
    match auth_service.authenticate_user(&request.email, &request.password).await {
        Ok (authenticated_user) => {
            Ok(HttpResponse::Ok()
                .cookie(CookieService::auth_cookie(&authenticated_user.token))
                .json(serde_json::json!({
                    "message": "Login successful",
                    "success": true,
                })))
        }
        Err (e) => Err(
            actix_web::error::ErrorUnauthorized(e)
        ),
    }
}

#[get("/me")]
async fn get_current_user(
    state: web::Data<AppState>,
    claims: Claims,
) -> Result<impl Responder> {
    let user_service = UserService::new(state.as_ref().clone());
    
    match user_service.get_user_by_id(&claims.sub).await {
        Ok(Some(user)) => Ok(HttpResponse::Ok().json(MeResponse {
            user: UserResponse {
                id: user.id.to_string(),
                email: user.email,
                username: user.username.unwrap_or_default(),
                roles: claims.roles,
                permissions: claims.permissions,
            },
            token_purpose: claims.purpose,
            issued_at: claims.iat as i64,
        })),
        Ok(None) => Err(actix_web::error::ErrorNotFound("User not found")),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}

//Register Route
#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    #[serde(rename = "confirmPassword")]
    pub confirm_password: String,
    pub email: String,
}

#[post("/register")]
pub async fn register(
    state: web::Data<AppState>,
    request: web::Json<RegisterRequest>,
) -> Result<impl Responder> {
    let request = request.into_inner();
    let auth_service = AuthService::new(state.as_ref().clone());
    
    match auth_service.register_user(request).await {
        Ok(_) => Ok(HttpResponse::Ok().json(
            serde_json::json!({
                "message": "User registered successfully",
                "success": true
            }
        ))),
        Err(e) => Err(actix_web::error::ErrorBadRequest(e)),
    }
}

//Logout Route
#[post("/logout")]
async fn logout() -> Result<impl Responder> {

    let logout_cookie = CookieService::logout_cookie();

    Ok(HttpResponse::Ok()
        .cookie(logout_cookie)
        .json(serde_json::json!({
            "message": "Logged out successfully",
            "success": true
        })))
}


//Oauth2 Route
#[derive(Deserialize)]
struct Oauth2Request {
    pub provider: String,
    pub code: String,
}

