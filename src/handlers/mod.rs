pub mod auth_handler;
pub mod health_handler;
pub mod mfa_handler;

pub fn configure_unprotected_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(health_handler::health_check);
}

pub fn configure_protected_routes(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(auth_handler::login)
        .service(auth_handler::logout)
        .service(auth_handler::refresh_token)
        .service(mfa_handler::enable_mfa)
        .service(mfa_handler::disable_mfa)
        .service(mfa_handler::verify_mfa);
}