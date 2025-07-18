use sea_orm::{DerivePrimaryKey, Set};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "token_blacklist")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub jti: String,
    pub user_id: Uuid,
    pub token_type: String,
    pub blacklisted_at: ChronoDateTimeUtc,
    pub expires_at: ChronoDateTimeUtc,
    pub reason: Option<String>, // "logout", "consumed", "revoked", etc.
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id",
        on_update = "NoAction",
        on_delete = "Cascade"
    )]
    Users,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            jti: Set(Uuid::new_v4().to_string()),
            user_id: Set(Uuid::nil()),
            token_type: Set("access".to_owned()),
            blacklisted_at: Set(chrono::Utc::now()),
            expires_at: Set(chrono::Utc::now() + chrono::Duration::days(30)),
            reason: Set(None),
        }
    }
}