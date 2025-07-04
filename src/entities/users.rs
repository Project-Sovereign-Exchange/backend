use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub username: Option<String>,
    pub password_hash: String,
    pub email: String,
    pub email_verified: bool,
    pub email_verified_at: Option<ChronoDateTimeUtc>,
    pub avatar_url: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone_number: Option<String>,
    pub phone_verified: bool,
    pub phone_verified_at: Option<ChronoDateTimeUtc>,
    pub totp_enabled: bool,
    pub totp_secret: Option<String>,
    pub account_status: String,
    pub last_login_at: Option<ChronoDateTimeUtc>,
    pub password_changed_at: Option<ChronoDateTimeUtc>,
    pub login_attempts: Option<i32>,
    pub locked_until: Option<ChronoDateTimeUtc>,
    pub timezone: Option<String>,
    pub language: Option<String>,
    pub date_of_birth: Option<ChronoDate>,
    pub address_line_1: Option<String>,
    pub address_line_2: Option<String>,
    pub city: Option<String>,
    pub state_province: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub address_latitude: Option<Decimal>,
    pub address_longitude: Option<Decimal>,
    pub address_verified: bool,
    pub address_verified_at: Option<ChronoDateTimeUtc>,
    pub address_verification_method: Option<String>,
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::mfa_backup_codes::Entity")]
    MfaBackupCodes,
    #[sea_orm(has_many = "super::known_login_locations::Entity")]
    KnownLoginLocations,
    #[sea_orm(has_many = "super::login_history::Entity")]
    LoginHistory,
    #[sea_orm(has_many = "super::user_sessions::Entity")]
    UserSessions,
    #[sea_orm(has_many = "super::address_history::Entity")]
    AddressHistory,
}

impl Related<super::mfa_backup_codes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::MfaBackupCodes.def()
    }
}

impl Related<super::known_login_locations::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::KnownLoginLocations.def()
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

impl Related<super::address_history::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AddressHistory.def()
    }
}

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: Set(Uuid::new_v4()),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            email_verified: Set(false),
            phone_verified: Set(false),
            totp_enabled: Set(false),
            account_status: Set("active".to_owned()),
            login_attempts: Set(Some(0)),
            timezone: Set(Some("UTC".to_owned())),
            language: Set(Some("en".to_owned())),
            address_verified: Set(false),
            password_changed_at: Set(Some(chrono::Utc::now())),
            ..ActiveModelTrait::default()
        }
    }
}