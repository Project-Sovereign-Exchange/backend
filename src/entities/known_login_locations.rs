use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "known_login_locations")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub ip_address: String,
    pub country: Option<String>,
    pub region: Option<String>,
    pub city: Option<String>,
    pub latitude: Option<Decimal>,
    pub longitude: Option<Decimal>,
    pub timezone: Option<String>,
    pub isp: Option<String>,
    pub user_agent: Option<String>,
    pub device_fingerprint: Option<String>,
    pub is_trusted: bool,
    pub trust_level: i32,
    pub first_seen_at: ChronoDateTimeUtc,
    pub last_seen_at: ChronoDateTimeUtc,
    pub login_count: i32,
    pub is_active: bool,
    pub notes: Option<String>,
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
    #[sea_orm(has_many = "super::login_history::Entity")]
    LoginHistory,
    #[sea_orm(has_many = "super::user_sessions::Entity")]
    UserSessions,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl Related<super::login_history::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::LoginHistory.def()
    }
}

impl Related<super::user_sessions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UserSessions.def()
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: Set(Uuid::new_v4()),
            is_trusted: Set(false),
            trust_level: Set(0),
            first_seen_at: Set(chrono::Utc::now()),
            last_seen_at: Set(chrono::Utc::now()),
            login_count: Set(1),
            is_active: Set(true),
            ..ActiveModelTrait::default()
        }
    }
}