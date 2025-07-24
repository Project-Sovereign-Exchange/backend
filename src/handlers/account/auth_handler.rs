use actix_web::{get, post, web, HttpResponse, Responder, Result};
use actix_web::cookie::{Cookie, SameSite};
use actix_web::http::header;
use futures_util::TryFutureExt;
use serde::{Deserialize, Serialize};
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
struct LoginResponse {
    pub user_id: uuid::Uuid,
    pub username: String,
    pub email: String,
}

#[post("/login")]
async fn login(
    state: web::Data<AppState>,
    request: web::Json<LoginRequest>,
) -> Result<impl Responder> {
    let auth_service = AuthService::new(state.as_ref().clone());
    
    match auth_service.authenticate_user(&request.email, &request.password).await {
        Ok (authenticated_user) => {
            Ok(actix_web::HttpResponse::Ok()
                .cookie(CookieService::auth_cookie(&authenticated_user.token))
                .json(LoginResponse {
                    user_id: authenticated_user.user.id,
                    username: authenticated_user.user.username.unwrap_or_default(),
                    email: authenticated_user.user.email,
                }))
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
        Ok(Some(user)) => Ok(HttpResponse::Ok().json(LoginResponse {
            user_id: user.id,
            username: user.username.unwrap_or_default(),
            email: user.email,
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
        Ok(_) => Ok(HttpResponse::Ok()),
        Err(e) => Err(actix_web::error::ErrorBadRequest(e)),
    }
}

//Logout Route
#[post("/logout")]
async fn logout() -> Result<impl Responder> {
    // Invalidate the user's session or token here
    Ok(actix_web::HttpResponse::Ok().json("User logged out successfully"))
}


//Oauth2 Route
#[derive(Deserialize)]
struct Oauth2Request {
    pub provider: String,
    pub code: String,
}

