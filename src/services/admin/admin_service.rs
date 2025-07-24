use sea_orm::*;
use uuid::Uuid;
use crate::app_state::AppState;
use crate::entities::admin_users;

pub struct AdminService {
    state: AppState,
}

impl AdminService {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
    
    pub async fn get_admin_by_id(
        &self,
        admin_id: &Uuid,
    ) -> Result<Option<admin_users::Model>, String> {
        admin_users::Entity::find_by_id(*admin_id)
            .one(&self.state.db)
            .await
            .map_err(|e| format!("Failed to fetch admin: {}", e))
    }
    
    pub async fn get_admin_by_email(
        &self,
        email: &str,
    ) -> Result<Option<admin_users::Model>, String> {
        admin_users::Entity::find()
            .filter(admin_users::Column::Email.eq(email))
            .one(&self.state.db)
            .await
            .map_err(|e| format!("Failed to fetch admin by email: {}", e))
    }
}