use actix_web::{post, web, Responder, Result};
use crate::app_state::AppState;
use crate::services::account::auth_service::AuthService;
use crate::services::integrations::cookie_service::CookieService;

#[derive(serde::Deserialize)]
struct AdminLoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(serde::Serialize)]
struct AdminLoginResponse {
    pub user_id: uuid::Uuid,
    pub username: String,
    pub email: String,
}

#[post("/login")]
async fn login(
    state: web::Data<AppState>,
    request: web::Json<AdminLoginRequest>,
) -> Result<impl Responder> {
    let auth_service = AuthService::new(state.as_ref().clone());

    match auth_service.authenticate_admin(&request.email, &request.password).await {
        Ok (authenticated_user) => {
            Ok(actix_web::HttpResponse::Ok()
                .cookie(CookieService::auth_cookie(&authenticated_user.token))
                .json(AdminLoginResponse {
                    user_id: authenticated_user.user.id,
                    username: authenticated_user.user.username,
                    email: authenticated_user.user.email,
                }))
        }
        Err (e) => Err(
            actix_web::error::ErrorUnauthorized(e)
        ),
    }
}