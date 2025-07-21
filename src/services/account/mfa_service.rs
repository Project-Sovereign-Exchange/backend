use actix_web::HttpResponse;
use sea_orm::*;
use totp_rs::Secret;
use uuid::Uuid;
use crate::app_state::AppState;
use crate::services::account::user_service::UserService;
use crate::entities::mfa_backup_codes;
use crate::handlers::account::mfa_handler::{MFALoginRequest, MFARequest};

pub struct MfaService {
    state: AppState,
}

impl MfaService {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    pub async fn generate_user_secret(
        &self,
        user_id: &Uuid
    ) -> Result<(), String> {
        let secret = Secret::generate_secret();
        let secret_base32 = secret.to_encoded().to_string();

        let user_service = UserService::new(self.state.clone());

        let user = user_service.update_user_field(
            user_id,
            |user| {
                user.totp_secret = Set(Some(secret_base32.clone()));
            }
        ).await.map_err(|e| format!("Failed to update user secret: {}", e))?;
        
        Ok(())
    }
    
    pub async fn generate_qr_code(
        &self,
        user_id: &Uuid
    ) -> Result<String, Box<dyn std::error::Error>> {
        let user_service = UserService::new(self.state.clone());

        let user = user_service.get_user_by_id(user_id).await
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
    
    pub async fn verify_totp_code(
        &self,
        user_email: &str,
        totp_code: &str,
        secret_base32: &str
    ) -> Result<bool, String> {
        let secret = Secret::Encoded(secret_base32.to_string());
        
        let totp = totp_rs::TOTP::new(
            totp_rs::Algorithm::SHA1,
            6,
            1,
            30,
            secret.to_bytes()
            .map_err(|e| format!("Failed to parse code: {}", e))?,
            Some("TCGEmporium".to_string()),
            user_email.to_string(),
        ).map_err(|e| format!("Failed to generate totp: {}", e))?;

        let is_valid = totp.check_current(totp_code)
            .map_err(|e| format!("Failed to validate totp: {}", e))?;
        
        Ok(is_valid)
    }
    
    pub async fn verify_backup_code(
        &self,
        user_id: &Uuid, backup_code: &str
    ) -> Result<bool, String> {
        let backup_codes = self.get_backup_codes(&user_id)
            .await
            .map_err(|e| format!("Failed to fetch backup codes: {}", e))?;
        
        for code in backup_codes {
            let code_hashed = &code.code_hash;
            
            let correct = bcrypt::verify(backup_code, &code_hashed)
                .map_err(|e| format!("Failed to verify backup code: {}", e))?;
            
            if correct && code.used_at.is_none() {
                let db = &self.state.db;
                
                let mut active_code: mfa_backup_codes::ActiveModel = code.into();
                active_code.used_at = Set(Some(chrono::Utc::now()));
                
                active_code.update(db).await
                    .map_err(|e| format!("Failed to update backup code: {}", e))?;
                
                return Ok(true);
            }
        }
        
        Err("Invalid backup code".to_string())
    }

    pub async fn enable_mfa_for_user(
        &self,
        user_id: &Uuid,
        code: &str
    ) -> Result<(), String> {
        let user_service = UserService::new(self.state.clone());

        let user = user_service.get_user_by_id(&user_id)
            .await
            .map_err(|e| format!("Failed to fetch user: {}", e))?
            .ok_or_else(||"User not found")?;

        if user.totp_enabled {
            return Err("MFA is already enabled for this user".to_string());
        }

        let totp_secret = user.totp_secret
            .as_ref()
            .ok_or_else(|| "MFA secret not set for user")?;
        
        self.verify_totp_code(&user.email, code, totp_secret)
            .await
            .map_err(|e| format!("Failed to verify TOTP code: {}", e))?;
        
        user_service.update_user_field(
            &user_id,
            |user| {
                user.totp_enabled = Set(true);
            }
        )
        .await
        .map_err(|e| format!("Failed to enable MFA for user: {}", e))?;
        
        Ok(())
    }
    
    pub async fn disable_mfa_for_user(
        &self,
        user_id: &Uuid,
        request: MFARequest,
    ) -> Result<(), String> {
        let user_service = UserService::new(self.state.clone());

        let user = user_service.get_user_by_id(&user_id)
            .await
            .map_err(|e| format!("Failed to fetch user: {}", e))?
            .ok_or_else(|| "User not found")?;

        if !user.totp_enabled {
            return Err("MFA is not enabled for this user".to_string());
        }

        let totp_secret = user.totp_secret
            .as_ref()
            .ok_or_else(|| "MFA secret not set for user")?;
        
        self.verify_totp_code(&user.email, &request.mfa_code, totp_secret)
            .await
            .map_err(|e| format!("Failed to verify TOTP code: {}", e))?;
        
        user_service.update_user_field(
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
    
    async fn generate_backup_codes(
        &self,
        user_id: &Uuid
    ) -> Result<(), String> {
        let db = &self.state.db;
        
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
    
    async fn get_backup_codes(
        &self,
        user_id: &Uuid
    ) -> Result<Vec<mfa_backup_codes::Model>, String> {
        let db = &self.state.db;
        
        let backup_codes = mfa_backup_codes::Entity::find()
            .filter(mfa_backup_codes::Column::UserId.eq(*user_id))
            .all(db)
            .await
            .map_err(|e| format!("Failed to fetch backup codes: {}", e))?;
        
        Ok(backup_codes)
    }
    
    async fn clear_backup_codes(
        &self,
        user_id: &Uuid
    ) -> Result<(), String> {
        let db = &self.state.db;
        
        mfa_backup_codes::Entity::delete_many()
            .filter(mfa_backup_codes::Column::UserId.eq(*user_id))
            .exec(db)
            .await
            .map_err(|e| format!("Failed to clear backup codes: {}", e))?;
        
        Ok(())
    }

    pub async fn verify_code(
        &self,
        user_id: &Uuid,
        request: MFALoginRequest
    ) -> Result<(), String> {
        let user_service = UserService::new(self.state.clone());
        let user = user_service.get_user_by_id(&user_id)
            .await
            .map_err(|e| format!("Failed to fetch user: {}", e))?
            .ok_or_else(|| "User not found")?;

        if let Some(secret) = user.totp_secret {
            let is_valid = if request.is_backup_code {
                self.verify_backup_code(
                    &user.id,
                    &request.mfa_code
                ).await
            } else {
                self.verify_totp_code(
                    &user.email,
                    &request.mfa_code,
                    &secret
                ).await
            };

            match is_valid {
                Ok(true) => Ok(()),
                Ok(false) => Err("MFA code is invalid".to_string()),
                Err(e) => Err(format!("Failed to verify code: {}", e)),
            }

        } else {
            Err("MFA is not enabled for this user".to_string())
        }
    }
}