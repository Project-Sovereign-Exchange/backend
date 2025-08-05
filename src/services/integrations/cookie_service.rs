use actix_web::cookie::{Cookie, SameSite};
use actix_web::cookie::time::Duration;

pub struct CookieService;

impl CookieService {
    pub fn auth_cookie(token: &str) -> Cookie<'static> {
        Cookie::build("auth_token".to_string(), token.to_string())
            .domain("localhost")
            .http_only(true)
            .secure(true)
            .same_site(SameSite::None)
            .path("/")
            .max_age(Duration::hours(3))
            .finish()
    }

    pub fn logout_cookie() -> Cookie<'static> {
        Cookie::build("auth_token".to_string(), String::new())
            .domain("localhost")
            .http_only(true)
            .secure(true)
            .same_site(SameSite::None)
            .path("/")
            .max_age(Duration::seconds(-1))
            .finish()
    }
}
