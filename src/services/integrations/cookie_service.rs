use actix_web::cookie::{Cookie, SameSite};

pub struct CookieService;

impl CookieService {
    pub fn auth_cookie(token: &str) -> Cookie {
        Cookie::build("auth_token", token)
            .http_only(true)
            .secure(false)
            .same_site(SameSite::Lax)
            .domain("127.0.0.1")
            .path("/")
            .finish()
    }
}