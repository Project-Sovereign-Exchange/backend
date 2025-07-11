use actix_web::{post, web, Responder, Result};
use actix_web::cookie::{Cookie, SameSite};
use actix_web::http::header;
use futures_util::TryFutureExt;
use serde::{Deserialize, Serialize};
use crate::services::auth_service::AuthService;
use crate::services::jwt_service::JwtService;
use crate::services::user_service::UserService;
use crate::utils::validator_util::ValidatorUtil;

//Login Route
#[derive(Deserialize)]
struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    pub user_id: uuid::Uuid,
    pub username: String,
    pub email: String,
}

#[post("/login")]
pub async fn login_user(
    request: web::Json<LoginRequest>,
) -> Result<impl Responder> {
    
    let authenticated_user = AuthService::authenticate_user(
        &request.username, 
        &request.password
    ).await.map_err(|e| actix_web::error::ErrorBadRequest(e))?;
    
    let cookie = Cookie::build("auth_token", authenticated_user.token)
        .http_only(true)
        .secure(false)
        .same_site(SameSite::Strict)
        .finish();
    
    Ok(actix_web::HttpResponse::Ok()
        .cookie(cookie)
        .json(LoginResponse {
            user_id: authenticated_user.user.id,
            username: authenticated_user.user.username.unwrap_or_default(),
            email: authenticated_user.user.email,
        }))
}



//Register Route
#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub email: String,
}

#[post("/register")]
pub async fn register_user(
    request: web::Json<RegisterRequest>,
) -> Result<impl Responder> {
    
    if !ValidatorUtil::validate_email(&request.email) {
        return Err(actix_web::error::ErrorBadRequest("Invalid email format"));
    }
    
    if !ValidatorUtil::validate_password(&request.password) {
        return Err(actix_web::error::ErrorBadRequest("Invalid password format"));
    }
    
    let user = UserService::create_user(
        request.into_inner(),
    ).await.map_err(|e| actix_web::error::ErrorBadRequest(e))?;

    Ok(actix_web::HttpResponse::Created().json("User registered successfully"))
}

//Logout Route
#[post("/logout")]
async fn logout_user() -> Result<impl Responder> {
    // Invalidate the user's session or token here
    Ok(actix_web::HttpResponse::Ok().json("User logged out successfully"))
}


//Oauth2 Route
#[derive(Deserialize)]
struct Oauth2Request {
    pub provider: String,
    pub code: String,
}

