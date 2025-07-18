use sea_orm::*;
use totp_rs::Secret;
use uuid::Uuid;
use crate::services::account::user_service::UserService;
use crate::services::integrations::db_service::DbService;
use crate::entities::mfa_backup_codes;

pub struct MFAService;

impl MFAService {
    pub async fn generate_user_secret(user_id: &Uuid) -> Result<(), String> {
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
    
    pub async fn generate_qr_code(user_id: &Uuid) -> Result<String, Box<dyn std::error::Error>> {
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
    
    pub async fn verify_totp_code(user_email: &str, totp_code: &str, secret_base32: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let secret = Secret::Encoded(secret_base32.to_string());
        
        let totp = totp_rs::TOTP::new(
            totp_rs::Algorithm::SHA1,
            6,
            1,
            30,
            secret.to_bytes()?,
            Some("TCGEmporium".to_string()),
            user_email.to_string(),
        )?;

        let is_valid = totp.check_current(totp_code)?;
        
        Ok(is_valid)
    }
    
    pub async fn verify_backup_code(user_id: &Uuid, backup_code: &str) -> Result<bool, String> {
        let backup_codes = Self::get_backup_codes(&user_id)
            .await
            .map_err(|e| format!("Failed to fetch backup codes: {}", e))?;
        
        for code in backup_codes {
            let code_hashed = &code.code_hash;
            
            let correct = bcrypt::verify(backup_code, &code_hashed)
                .map_err(|e| format!("Failed to verify backup code: {}", e))?;
            
            if correct && code.used_at.is_none() {
                let db = DbService::get().await
                    .map_err(|e| format!("Failed to get database connection: {}", e))?;
                
                let mut active_code: mfa_backup_codes::ActiveModel = code.into();
                active_code.used_at = Set(Some(chrono::Utc::now()));
                
                active_code.update(db).await
                    .map_err(|e| format!("Failed to update backup code: {}", e))?;
                
                return Ok(true);
            }
        }
        
        Err("Invalid backup code".to_string())
    }

    pub async fn enable_mfa_for_user(user_id: &Uuid, code: &str) -> Result<(), String> {
        let user = UserService::get_user_by_id(&user_id)
            .await
            .map_err(|e| format!("Failed to fetch user: {}", e))?
            .ok_or_else(||"User not found")?;

        if user.totp_enabled {
            return Err("MFA is already enabled for this user".to_string());
        }

        let totp_secret = user.totp_secret
            .as_ref()
            .ok_or_else(|| "MFA secret not set for user")?;
        
        Self::verify_totp_code(&user.email, code, totp_secret)
            .await
            .map_err(|e| format!("Failed to verify TOTP code: {}", e))?;
        
        UserService::update_user_field(
            &user_id,
            |user| {
                user.totp_enabled = Set(true);
            }
        )
        .await
        .map_err(|e| format!("Failed to enable MFA for user: {}", e))?;
        
        Ok(())
    }
    
    pub async fn disable_mfa_for_user(user_id: &Uuid, code: &str) -> Result<(), String> {
        let user = UserService::get_user_by_id(&user_id)
            .await
            .map_err(|e| format!("Failed to fetch user: {}", e))?
            .ok_or_else(|| "User not found")?;

        if !user.totp_enabled {
            return Err("MFA is not enabled for this user".to_string());
        }

        let totp_secret = user.totp_secret
            .as_ref()
            .ok_or_else(|| "MFA secret not set for user")?;
        
        Self::verify_totp_code(&user.email, code, totp_secret)
            .await
            .map_err(|e| format!("Failed to verify TOTP code: {}", e))?;
        
        UserService::update_user_field(
            &user_id,
            |user| {
                user.totp_enabled = Set(false);
                user.totp_secret = Set(None);
            }
        )
        .await
        .map_err(|e| format!("Failed to disable MFA for user: {}", e))?;
        
        Ok(())
    }
    
    async fn generate_backup_codes(user_id: &Uuid) -> Result<(), String> {
        let db = DbService::get().await
            .map_err(|e| format!("Failed to get database connection: {}", e))?;
        
        let mut codes = Vec::new();
        
        for i in 0..8 {
            let code = format!("{:08}", rand::random::<u32>() % 100_000_000);
            let code_hash = bcrypt::hash(&code, 4)
                .map_err(|e| format!("Failed to hash backup code: {}", e))?;
            
            let backup_code = mfa_backup_codes::ActiveModel {
                id: Set(Uuid::new_v4()),
                user_id: Set(*user_id),
                code_hash: Set(code_hash),
                label: Set(Some(format!("Backup Code {}", i + 1))),
                is_used: Set(false),
                used_at: Set(None),
                created_at: Set(chrono::Utc::now()),
                expires_at: Set(Some(chrono::Utc::now() + chrono::Duration::days(90))),
            };
            
            let inserted_code = backup_code.insert(db).await
                .map_err(|e| format!("Failed to insert backup code: {}", e))?;
            
            codes.push(inserted_code);
        }
        
        Ok(())
    }
    
    async fn get_backup_codes(user_id: &Uuid) -> Result<Vec<mfa_backup_codes::Model>, String> {
        let db = DbService::get().await
            .map_err(|e| format!("Failed to get database connection: {}", e))?;
        
        let backup_codes = mfa_backup_codes::Entity::find()
            .filter(mfa_backup_codes::Column::UserId.eq(*user_id))
            .all(db)
            .await
            .map_err(|e| format!("Failed to fetch backup codes: {}", e))?;
        
        Ok(backup_codes)
    }
    
    async fn clear_backup_codes(user_id: &Uuid) -> Result<(), String> {
        let db = DbService::get().await
            .map_err(|e| format!("Failed to get database connection: {}", e))?;
        
        mfa_backup_codes::Entity::delete_many()
            .filter(mfa_backup_codes::Column::UserId.eq(*user_id))
            .exec(db)
            .await
            .map_err(|e| format!("Failed to clear backup codes: {}", e))?;
        
        Ok(())
    }
}