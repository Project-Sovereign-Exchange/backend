use std::env;
use config::Config;
use jsonwebtoken::{decode, DecodingKey, Validation, encode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: usize,
    pub iat: usize,
}

pub struct JwtService {}

impl JwtService {

    pub fn generate_token(user_id: Uuid) -> Result<String, String> {
        
        let claims = Claims {
            sub: user_id,
            exp: (chrono::Utc::now() + chrono::Duration::days(7)).timestamp() as usize,
            iat: chrono::Utc::now().timestamp() as usize,
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

    pub fn validate_token(token: &str) -> Result<Uuid, String> {
        let secret = env::var("JWT_SECRET")
            .map_err(|_| "JWT_SECRET must be set")?;

        let token = decode::<Claims>(token, &DecodingKey::from_secret(secret.as_ref()), &Validation::default())
            .map_err(|e| format!("Invalid token: {}", e))?;
        
        Ok(token.claims.sub)
    }
}