use actix_web::web;
use sea_orm::*;
use uuid::Uuid;
use chrono::Utc;
use crate::app_state::AppState;
use crate::entities::users;
use crate::handlers::account::auth_handler::RegisterRequest;

pub struct UserService {
    state: AppState,
}

impl UserService {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
    
    pub async fn create_user(
        &self,
        request: RegisterRequest,
    ) -> Result<users::Model, String> {
        if let Some(existing_user) = self.get_user_by_email(&request.email).await? {
            return Err("Email already in use".to_string());
        }
        
        if let Some(existing_user) = self.get_user_by_username(&request.username).await? {
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

        let user = user.insert(&self.state.db).await
            .map_err(|e| format!("Failed to create user: {}", e))?;
        
        Ok(user)
    }
    
    pub async fn get_user_by_id(
        &self,
        user_id: &Uuid,
    ) -> Result<Option<users::Model>, String> {
        users::Entity::find_by_id(*user_id)
            .one(&self.state.db)
            .await
            .map_err(|e| format!("Failed to fetch user: {}", e))
    }
    
    pub async fn get_user_by_email(
        &self,
        email: &str,
    ) -> Result<Option<users::Model>, String> {
        users::Entity::find()
            .filter(users::Column::Email.eq(email))
            .one(&self.state.db)
            .await
            .map_err(|e| format!("Failed to fetch user by email: {}", e))
    }
    
    pub async fn get_user_by_username(
        &self,
        username: &str,
    ) -> Result<Option<users::Model>, String> {
        users::Entity::find()
            .filter(users::Column::Username.eq(username))
            .one(&self.state.db)
            .await
            .map_err(|e| format!("Failed to fetch user by username: {}", e))
    }

    pub async fn update_user_field<F>(
        &self,
        user_id: &Uuid,
        update_fn: F,
    ) -> Result<users::Model, String>
    where
        F: FnOnce(&mut users::ActiveModel),
    {
        let user = users::Entity::find_by_id(*user_id)
            .one(&self.state.db)
            .await
            .map_err(|e| format!("Failed to fetch user: {}", e))?
            .ok_or_else(|| "User not found".to_string())?;

        let mut user_update: users::ActiveModel = user.into();

        user_update.updated_at = Set(Utc::now());

        update_fn(&mut user_update);

        let updated_user = user_update
            .update(&self.state.db)
            .await
            .map_err(|e| format!("Failed to update user: {}", e))?;
        
        Ok(updated_user)
    }
}