use sea_orm::*;
use uuid::Uuid;
use crate::app_state::AppState;
use crate::entities::{user_roles};
use crate::entities::user_roles::UserRoleType;

pub struct AdminService {
    state: AppState,
}

impl AdminService {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    pub async fn is_admin(
        &self,
        user_id: &Uuid
    ) -> Result<bool, String> {
        let admin_role = user_roles::Entity::find()
            .filter(user_roles::Column::UserId.eq(*user_id))
            .filter(user_roles::Column::Role.eq(UserRoleType::Admin))
            .filter(user_roles::Column::IsActive.eq(true))
            .one(&self.state.db)
            .await
            .map_err(|e| format!("Failed to check user role: {}", e))?;

        Ok(admin_role.is_some())
    }

    pub async fn is_moderator(
        &self,
        user_id: &Uuid
    ) -> Result<bool, String> {
        let admin_role = user_roles::Entity::find()
            .filter(user_roles::Column::UserId.eq(*user_id))
            .filter(user_roles::Column::Role.eq(UserRoleType::Moderator))
            .filter(user_roles::Column::IsActive.eq(true))
            .one(&self.state.db)
            .await
            .map_err(|e| format!("Failed to check user role: {}", e))?;

        Ok(admin_role.is_some())
    }
}