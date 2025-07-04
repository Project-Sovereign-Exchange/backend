use actix_web::{post, web, Responder, Result};
use serde::{Deserialize, Serialize};

//Login Route
#[derive(Deserialize)]
struct LoginRequest {
    pub username: String,
    pub password: String,
}



//Register Route
#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    pub email: String,
}

#[post("/register")]
async fn register_user(
    request: web::Json<RegisterRequest>,
) -> Result<impl Responder> {
    
    
    
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

//MFA code validation route
#[derive(Deserialize)]
struct MFARequest {
    pub mfa_code: String,
    pub is_backup_code: bool,
}

//Setup MFA route
//Check if a user has MFA enabled, if they do return error, if not, enable MFA
//Just set the secret and return the QR code URL, do not enable MFA yet

//Enable MFA route
//Validate the totp against the secret and enable MFA for the user

//Configure routes
