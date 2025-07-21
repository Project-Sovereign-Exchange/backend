use serde::Deserialize;
use actix_web::{web, HttpResponse, Responder, post};
use sea_orm::Set;
use crate::app_state::AppState;
use crate::entities::users::Model;
use crate::services::account::jwt_service::Claims;
use crate::services::account::mfa_service::{MfaService};
use crate::services::account::user_service::UserService;
use crate::services::marketplace::listing_service::ListingService;

//MFA code validation route
#[derive(Deserialize)]
pub struct MFARequest {
    pub mfa_code: String,
    pub is_backup_code: bool,
}

#[derive(Deserialize)]
pub struct MFALoginRequest {
    pub mfa_code: String,
    pub is_backup_code: bool,
    pub temp_token: String,
}

//Setup MFA route
//Check if a user has MFA enabled, if they do return error, if not, enable MFA
//Just set the secret and return the QR code URL, do not enable MFA yet
#[post("/setup")]
async fn setup_mfa(
    state: web::Data<AppState>,
    claims: Claims,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = claims.sub;
    let mfa_service = MfaService::new(state.as_ref().clone());
    
    match mfa_service.generate_user_secret(&user_id).await {
        Ok(_) => {}
        Err(e) => {
            Err(actix_web::error::ErrorInternalServerError(e))?;
        }
    }
    
    match mfa_service.generate_qr_code(&user_id).await {
        Ok(qr_code_url) => Ok(HttpResponse::Ok().json(qr_code_url)),
        Err(e) => {
            Err(actix_web::error::ErrorInternalServerError(e))
        }
    }
}

//Enable MFA route
#[post("/enable")]
async fn enable_mfa(
    state: web::Data<AppState>,
    claims: Claims,
    request: web::Json<MFARequest>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = claims.sub;
    let mfa_code = &request.mfa_code;
    let mfa_service = MfaService::new(state.as_ref().clone());
    
    match mfa_service.enable_mfa_for_user(&user_id, &mfa_code).await {
        Ok(_) => Ok(HttpResponse::Ok().finish()),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}

#[post("/verify")]
async fn verify_mfa(
    state: web::Data<AppState>,
    claims: Claims,
    request: web::Json<MFALoginRequest>,
) -> Result<impl Responder, actix_web::Error> {
    let user_id = claims.sub;
    let request = request.into_inner();
    let mfa_service = MfaService::new(state.as_ref().clone());
    
    match mfa_service.verify_code(&user_id, request).await {
        Ok(_) => Ok(HttpResponse::Ok()),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}

#[post("/disable")]
async fn disable_mfa(
    state: web::Data<AppState>,
    claims: Claims,
    request: web::Json<MFARequest>,
) -> Result<impl Responder, actix_web::Error> {
    let request = request.into_inner();
    let user_id = claims.sub;
    let mfa_service = MfaService::new(state.as_ref().clone());
    
    match mfa_service.disable_mfa_for_user(&user_id, request).await {
        Ok(_) => Ok(HttpResponse::Ok()),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}



//Validate the totp against the secret and enable MFA for the user

//Configure routes