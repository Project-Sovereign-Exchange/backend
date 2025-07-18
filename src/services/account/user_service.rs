use sea_orm::*;
use uuid::Uuid;
use chrono::Utc;
use crate::entities::users;
use crate::handlers::account::auth_handler::RegisterRequest;
use crate::services::integrations::db_service::DbService;

pub struct UserService;

impl UserService {
    pub async fn create_user(
        request: RegisterRequest,
    ) -> Result<users::Model, String> {
        
        let db = DbService::get().await
            .map_err(|e| format!("Failed to get database connection: {}", e))?;
        
        if let Some(existing_user) = UserService::get_user_by_email(&request.email).await? {
            return Err("Email already in use".to_string());
        }
        
        if let Some(existing_user) = UserService::get_user_by_username(db, &request.username).await? {
            return Err("Username already in use".to_string());
        }
        
        let password_hash = bcrypt::hash(&request.password, bcrypt::DEFAULT_COST)
            .map_err(|e| format!("Failed to hash password: {}", e))?;

        let user = users::ActiveModel {
            username: Set(Some(request.username)),
            email: Set(request.email),
            password_hash: Set(password_hash),
            created_at: Set(Utc::now()),
            updated_at: Set(Utc::now()),
            ..Default::default()
        };

        let user = user.insert(db).await
            .map_err(|e| format!("Failed to create user: {}", e))?;
        
        Ok(user)
    }
    
    pub async fn get_user_by_id(
        user_id: &Uuid,
    ) -> Result<Option<users::Model>, String> {
        let db = DbService::get().await
            .map_err(|e| format!("Failed to get database connection: {}", e))?;
        
        users::Entity::find_by_id(*user_id)
            .one(db)
            .await
            .map_err(|e| format!("Failed to fetch user: {}", e))
    }
    
    pub async fn get_user_by_email(
        email: &str,
    ) -> Result<Option<users::Model>, String> {
        let db = DbService::get().await
            .map_err(|e| format!("Failed to get database connection: {}", e))?;
        
        users::Entity::find()
            .filter(users::Column::Email.eq(email))
            .one(db)
            .await
            .map_err(|e| format!("Failed to fetch user by email: {}", e))
    }
    
    pub async fn get_user_by_username(
        db: &DatabaseConnection,
        username: &str,
    ) -> Result<Option<users::Model>, String> {
        users::Entity::find()
            .filter(users::Column::Username.eq(username))
            .one(db)
            .await
            .map_err(|e| format!("Failed to fetch user by username: {}", e))
    }

    pub async fn update_user_field<F>(
        user_id: &Uuid,
        update_fn: F,
    ) -> Result<users::Model, String>
    where
        F: FnOnce(&mut users::ActiveModel),
    {
        let db = DbService::get().await
            .map_err(|e| format!("Failed to get database connection: {}", e))?;
        
        let user = users::Entity::find_by_id(*user_id)
            .one(db)
            .await
            .map_err(|e| format!("Failed to fetch user: {}", e))?
            .ok_or_else(|| "User not found".to_string())?;

        let mut user_update: users::ActiveModel = user.into();

        user_update.updated_at = Set(Utc::now());

        update_fn(&mut user_update);

        let updated_user = user_update
            .update(db)
            .await
            .map_err(|e| format!("Failed to update user: {}", e))?;
        
        Ok(updated_user)
    }
}