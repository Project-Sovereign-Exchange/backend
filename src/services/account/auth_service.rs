use crate::entities::users;
use crate::services::account::jwt_service::JwtService;
use crate::services::account::user_service::UserService;
use crate::utils::validator_util::ValidatorUtil;

pub struct AuthenticatedUser {
    pub user: users::Model,
    pub token: String,
}

pub struct AuthService;

impl AuthService {

    pub async fn authenticate_user(email: &str, password: &str) -> Result<AuthenticatedUser, String> {
        let authenticated_user;
        
        if email.is_empty() || password.is_empty() {
            return Err("Username and password cannot be empty".to_string());
        }
        
        match ValidatorUtil::validate_email(email) {
            Ok(_) => {}
            Err(e) => return Err(e.to_string()),
        }

        match UserService::get_user_by_email(&email).await {
            Ok(Some(user)) => {
                if bcrypt::verify(password, &user.password_hash).unwrap_or(false) {
                    
                    let token = match user.totp_enabled {
                        true => JwtService::generate_temporary_token(user.id).await,
                        false => JwtService::generate_access_token(user.id).await,
                    }.map_err(|_| "Error generating token".to_string())?;
                    
                    authenticated_user = AuthenticatedUser {
                        user: user.clone(),
                        token,
                    };

                    Ok(authenticated_user)
                } else {
                    Err("Invalid username or password".to_string())
                }
            }
            Ok(None) => {
                Err("Invalid username or password".to_string())
            }
            Err(_) => {
                Err("Invalid username or password".to_string())
            }
        }
    }

    pub fn logout(&self, token: &str) -> Result<String, String> {
        // Implement logout logic here
        if token.is_empty() {
            return Err("Token cannot be empty".to_string());
        }
        // Simulate successful logout
        Ok("Logout successful".to_string())
    }
}