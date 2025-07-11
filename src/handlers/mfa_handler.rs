use serde::Deserialize;
use actix_web::{web, HttpResponse, Responder, post};
use crate::entities::users::Model;
use crate::services::jwt_service::Claims;
use crate::services::mfa_service::MFAService;
use crate::services::user_service::UserService;

//MFA code validation route
#[derive(Deserialize)]
struct MFARequest {
    pub mfa_code: String,
    pub is_backup_code: bool,
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
    let is_backup_code = request.is_backup_code;
    

    Ok(HttpResponse::Ok().json("MFA enabled successfully"))
}



//Validate the totp against the secret and enable MFA for the user

//Configure routes