use std::env;
use actix_web::{Error, FromRequest, HttpMessage, HttpRequest};
use actix_web::dev::Payload;
use config::Config;
use futures_util::future::{ready, Ready};
use jsonwebtoken::{decode, DecodingKey, Validation, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::app_state::AppState;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: Uuid,
    pub purpose: String,
    pub exp: usize,
    pub iat: usize,
    pub jti: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    scope: Option<Vec<String>>,
}

impl FromRequest for Claims {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        match req.extensions().get::<Claims>() {
            Some(claims) => ready(Ok(claims.clone())),
            None => ready(Err(actix_web::error::ErrorUnauthorized(
                "No authentication token found"
            ))),
        }
    }
}

pub struct JwtService {
    state: AppState,
}

impl JwtService {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
    
    pub async fn generate_admin_token(user_id: Uuid) -> Result<String, String> {
        let claims = Claims {
            sub: user_id,
            purpose: "admin".to_string(),
            exp: (chrono::Utc::now() + chrono::Duration::hours(3)).timestamp() as usize,
            iat: chrono::Utc::now().timestamp() as usize,
            jti: Uuid::new_v4().to_string(),
            scope: Some(vec!["admin".to_string()]),
        };
        
        let secret = env::var("JWT_SECRET")
            .map_err(|e| format!("JWT_SECRET must be set: {}", e))?;
        
        let token = encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(secret.as_ref())
        ).map_err(|e| e.to_string())?;
        
        Ok(token)
    }

    pub async fn generate_access_token(user_id: Uuid) -> Result<String, String> {
        
        let claims = Claims {
            sub: user_id,
            purpose: "access".to_string(),
            exp: (chrono::Utc::now() + chrono::Duration::hours(3)).timestamp() as usize,
            iat: chrono::Utc::now().timestamp() as usize,
            jti: Uuid::new_v4().to_string(),
            scope: None,
        };
        
        let secret = env::var("JWT_SECRET")
            .map_err(|e| format!("JWT_SECRET must be set: {}", e))?;
        
        let token = encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(secret.as_ref())
        ).map_err(|e| e.to_string())?;
        
        Ok(token)
    }
    
    pub async fn generate_temporary_token(user_id: Uuid) -> Result<String, String> {
        let claims = Claims {
            sub: user_id,
            purpose: "temporary".to_string(),
            exp: (chrono::Utc::now() + chrono::Duration::minutes(5)).timestamp() as usize,
            iat: chrono::Utc::now().timestamp() as usize,
            jti: Uuid::new_v4().to_string(),
            scope: None,
        };
        
        let secret = env::var("JWT_SECRET")
            .map_err(|e| format!("JWT_SECRET must be set: {}", e))?;
        
        let token = encode(
            &jsonwebtoken::Header::default(),
            &claims,
            &jsonwebtoken::EncodingKey::from_secret(secret.as_ref())
        ).map_err(|e| e.to_string())?;
        
        Ok(token)
    }

    pub async fn validate_token(token: &str) -> Result<Claims, String> {
        let secret = env::var("JWT_SECRET")
            .map_err(|_| "JWT_SECRET must be set")?;

        let token = decode::<Claims>(token, &DecodingKey::from_secret(secret.as_ref()), &Validation::default())
            .map_err(|e| format!("Invalid token: {}", e))?;
        
        Ok(token.claims)
    }
    
    pub async fn extract_user_id_from_token(token: &str) -> Result<Uuid, String> {
        let claims = Self::validate_token(token).await?;
        Ok(claims.sub)
    }
}