use actix_cors::Cors;
use std::env;

pub fn configure_cors() -> Cors {
    let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());

    match environment.as_str() {
        "production" => {
            Cors::default()
                .allowed_origin("https://yourdomain.com")
                .allowed_origin("https://www.yourdomain.com")
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                .allowed_headers(vec![
                    "Content-Type",
                    "Authorization",
                    "Accept",
                    "Origin",
                    "X-Requested-With",
                    "Cookie",
                ])
                .expose_headers(vec!["Set-Cookie"])
                .supports_credentials()
                .max_age(3600)
        }
        _ => {
            Cors::default()
                .allowed_origin("http://localhost:3000")
                .allowed_origin("http://127.0.0.1:3000")
                .allowed_origin("http://localhost:3001")
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                .allowed_headers(vec![
                    "Content-Type",
                    "Authorization",
                    "Accept",
                    "Origin",
                    "X-Requested-With",
                    "Access-Control-Request-Method",
                    "Access-Control-Request-Headers",
                    "Cookie",
                ])
                .expose_headers(vec!["Set-Cookie"])
                .supports_credentials()
                .max_age(300) // Shorter cache in development
        }
    }
}