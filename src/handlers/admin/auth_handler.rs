use actix_web::{get, post, web, HttpResponse, Responder, Result};
use crate::app_state::AppState;
use crate::services::account::auth_service::AuthService;
use crate::services::integrations::cookie_service::CookieService;
use crate::services::account::jwt_service::Claims;
use crate::services::admin::admin_service::AdminService;

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

#[get("/me")]
async fn get_current_admin(
    state: web::Data<AppState>,
    claims: Claims,
) -> Result<impl Responder> {
    let admin_service = AdminService::new(state.as_ref().clone());

    match admin_service.get_admin_by_id(&claims.sub).await {
        Ok(Some(admin)) => Ok(HttpResponse::Ok().json(AdminLoginResponse {
            user_id: admin.id,
            username: admin.username,
            email: admin.email,
        })),
        Ok(None) => Err(actix_web::error::ErrorNotFound("Admin not found")),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}
