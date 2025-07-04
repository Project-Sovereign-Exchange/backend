use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "user_sessions")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub session_token: String,
    pub refresh_token: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub location_id: Option<Uuid>,
    pub is_active: bool,
    pub created_at: ChronoDateTimeUtc,
    pub last_activity_at: ChronoDateTimeUtc,
    pub expires_at: ChronoDateTimeUtc,
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
    #[sea_orm(
        belongs_to = "super::known_login_locations::Entity",
        from = "Column::LocationId",
        to = "super::known_login_locations::Column::Id",
        on_update = "NoAction",
        on_delete = "SetNull"
    )]
    KnownLoginLocations,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl Related<super::known_login_locations::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::KnownLoginLocations.def()
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: Set(Uuid::new_v4()),
            is_active: Set(true),
            created_at: Set(chrono::Utc::now()),
            last_activity_at: Set(chrono::Utc::now()),
            expires_at: Set(chrono::Utc::now() + chrono::Duration::hours(24)),
            ..ActiveModelTrait::default()
        }
    }
}