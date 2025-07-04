use sea_orm::Set;
use totp_rs::Secret;
use uuid::Uuid;
use crate::services::user_service::UserService;

pub struct MFAService;

impl MFAService {
    pub async fn generate_user_secret(user_id: Uuid) -> Result<(), String> {
        let secret = Secret::generate_secret();
        let secret_base32 = secret.to_encoded().to_string();
        
        let user = UserService::update_user_field(
            user_id,
            |user| {
                user.totp_secret = Set(Some(secret_base32.clone()));
            }
        ).await.map_err(|e| format!("Failed to update user secret: {}", e))?;
        
        Ok(())
    }
    
    pub async fn generate_qr_code(user_id: Uuid) -> Result<String, Box<dyn std::error::Error>> {
        let user = UserService::get_user_by_id(user_id).await
            .map_err(|e| format!("Failed to fetch user: {}", e))?
            .ok_or("User not found")?;
        
        let user_label: &str = user.email.as_ref();
        
        let secret_base32: &str = user.totp_secret.as_ref()
            .ok_or("TOTP secret not set for user")?;
        
        let secret = Secret::Encoded(secret_base32.to_string());
        
        let totp = totp_rs::TOTP::new(
            totp_rs::Algorithm::SHA1,
            6,
            1,
            30,
            secret.to_bytes()?,
            Some("TCGEmporium".to_string()),
            user_label.to_string(),
        );
        
        let qr_code = totp.unwrap().get_qr_base64()
            .map_err(|e| format!("Failed to generate QR code: {}", e))?;
        
        Ok(qr_code)
    }
    
    pub async fn verify_totp_code(user_id: Uuid, totp_code: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let user = UserService::get_user_by_id(user_id).await
            .map_err(|e| format!("Failed to fetch user: {}", e))?
            .ok_or("User not found")?;
        
        let secret_base32 = user.totp_secret.as_ref()
            .ok_or("TOTP secret not set for user")?;
        
        let secret = Secret::Encoded(secret_base32.to_string());
        
        let totp = totp_rs::TOTP::new(
            totp_rs::Algorithm::SHA1,
            6,
            1,
            30,
            secret.to_bytes()?,
            Some("TCGEmporium".to_string()),
            user.email.clone(),
        )?;

        let is_valid = totp.check_current(totp_code)?;
        
        Ok(is_valid)
    }
}