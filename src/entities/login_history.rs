use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "login_history")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub device_fingerprint: Option<String>,
    pub login_method: String,
    pub success: bool,
    pub failure_reason: Option<String>,
    pub location_id: Option<Uuid>,
    pub session_id: Option<Uuid>,
    pub attempted_at: ChronoDateTimeUtc,
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
            login_method: Set("password".to_owned()),
            attempted_at: Set(chrono::Utc::now()),
            ..ActiveModelTrait::default()
        }
    }
}