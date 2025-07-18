use serde::Deserialize;
use actix_web::{web, HttpResponse, Responder, post};
use sea_orm::Set;
use crate::entities::users::Model;
use crate::services::account::jwt_service::Claims;
use crate::services::account::mfa_service::MFAService;
use crate::services::account::user_service::UserService;

//MFA code validation route
#[derive(Deserialize)]
struct MFARequest {
    pub mfa_code: String,
    pub is_backup_code: bool,
}

#[derive(Deserialize)]
struct MFALoginRequest {
    pub mfa_code: String,
    pub is_backup_code: bool,
    pub temp_token: String,
}

//Setup MFA route
//Check if a user has MFA enabled, if they do return error, if not, enable MFA
//Just set the secret and return the QR code URL, do not enable MFA yet
#[post("/setup")]
async fn setup_mfa(
    claims: Claims,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = claims.sub;
    // Generate TOTP secret for the user
    MFAService::generate_user_secret(&user_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    // Generate QR code URL for the TOTP secret
    let qr_code_url = MFAService::generate_qr_code(&user_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().json(qr_code_url))
}

//Enable MFA route
#[post("/enable")]
async fn enable_mfa(
    claims: Claims,
    request: web::Json<MFARequest>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = claims.sub;
    let mfa_code = &request.mfa_code;

    let user = UserService::get_user_by_id(&user_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
        .ok_or_else(|| actix_web::error::ErrorNotFound("User not found"))?;

    if let Some(secret) = user.totp_secret {
        let is_valid = MFAService::verify_totp_code(&user.email, mfa_code, &secret)
            .await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

        if is_valid {
            UserService::update_user_field(&user_id, |u| {
                u.totp_enabled = Set(true);
            })
                .await
                .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

            return Ok(HttpResponse::Ok().json("MFA enabled successfully"));
        }
    }
    
    Err(actix_web::error::ErrorBadRequest("Invalid MFA code"))
}

#[post("/verify")]
async fn verify_mfa(
    claims: Claims,
    request: web::Json<MFALoginRequest>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = claims.sub;
    let mfa_code = &request.mfa_code;
    let is_backup_code = request.is_backup_code;
    let temp_token = &request.temp_token;
    
    let user = UserService::get_user_by_id(&user_id)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
        .ok_or_else(|| actix_web::error::ErrorNotFound("User not found"))?;

    if let Some(secret) = user.totp_secret {
        let is_valid = if is_backup_code {
            MFAService::verify_backup_code(&user.id, mfa_code)
                .await
                .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
        } else {
            MFAService::verify_totp_code(&user.email, mfa_code, &secret)
                .await
                .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
        };

        if is_valid {
            return Ok(HttpResponse::Ok().json("MFA code validated successfully"));
        }
    }

    Err(actix_web::error::ErrorBadRequest("Invalid MFA code"))
}

// Disable MFA route
#[post("/disable")]
async fn disable_mfa(
    claims: Claims,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = claims.sub;

    // Disable MFA for the user
    UserService::update_user_field(&user_id, |u| {
        u.totp_enabled = Set(false);
        u.totp_secret = Set(None);
    })
    .await
    .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    Ok(HttpResponse::Ok().json("MFA disabled successfully"))
}



//Validate the totp against the secret and enable MFA for the user

//Configure routes