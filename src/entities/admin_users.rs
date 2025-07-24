use sea_orm::entity::prelude::*;
use sea_orm::Set;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "admin_users")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub username: String,
    pub password_hash: String,
    pub email: String,
    pub email_verified: bool,
    pub email_verified_at: Option<ChronoDateTimeUtc>,
    
    pub role: String, 
    pub permissions: Json,
    pub is_super_admin: bool,
    //useless for now, but will add support for "support" roles for ticket handling
    pub department: Option<String>,
    
    pub totp_enabled: bool,
    pub totp_secret: Option<String>,
    pub account_status: String, 
    pub last_login_at: Option<ChronoDateTimeUtc>,
    pub password_changed_at: Option<ChronoDateTimeUtc>,
    pub login_attempts: Option<i64>,
    pub locked_until: Option<ChronoDateTimeUtc>,
    pub require_password_change: bool,
    
    pub session_token: Option<String>,
    pub api_key: Option<String>,
    pub api_key_expires_at: Option<ChronoDateTimeUtc>,
    
    pub ip_whitelist: Option<Json>,
    pub access_level: i32,
    
    pub created_at: ChronoDateTimeUtc,
    pub updated_at: ChronoDateTimeUtc,
    pub created_by: Option<Uuid>,
    pub last_activity_at: Option<ChronoDateTimeUtc>,
    pub notes: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    /*
    #[sea_orm(has_many = "super::admin_mfa_backup_codes::Entity")]
    AdminMfaBackupCodes,
    #[sea_orm(has_many = "super::admin_login_history::Entity")]
    AdminLoginHistory,
    #[sea_orm(has_many = "super::admin_sessions::Entity")]
    AdminSessions,
    #[sea_orm(has_many = "super::admin_activity_log::Entity")]
    AdminActivityLog,
     */
    #[sea_orm(
        belongs_to = "Entity",
        from = "Column::CreatedBy",
        to = "Column::Id"
    )]
    CreatedByAdmin,
}

/*
Check which of these might be of importance in te future.

impl Related<super::admin_mfa_backup_codes::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AdminMfaBackupCodes.def()
    }
}

impl Related<super::admin_login_history::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AdminLoginHistory.def()
    }
}

impl Related<super::admin_sessions::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AdminSessions.def()
    }
}

impl Related<super::admin_activity_log::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AdminActivityLog.def()
    }
}
 */

impl ActiveModelBehavior for ActiveModel {
    fn new() -> Self {
        Self {
            id: Set(Uuid::new_v4()),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
            email_verified: Set(false),
            totp_enabled: Set(false),
            account_status: Set("active".to_owned()),
            login_attempts: Set(Some(0)),
            require_password_change: Set(true),
            password_changed_at: Set(Some(chrono::Utc::now())),
            is_super_admin: Set(false),
            role: Set("admin".to_owned()),
            permissions: Set(Json::Array(vec![])),
            access_level: Set(1),
            last_activity_at: Set(Some(chrono::Utc::now())),
            ..ActiveModelTrait::default()
        }
    }
}

impl Model {
    pub fn has_permission(&self, permission: &str) -> bool {
        if self.is_super_admin {
            return true;
        }

        if let Json::Array(perms) = &self.permissions {
            perms.iter().any(|p| {
                if let Json::String(perm_str) = p {
                    perm_str == permission
                } else {
                    false
                }
            })
        } else {
            false
        }
    }
    
    pub fn has_access_level(&self, required_level: i32) -> bool {
        self.access_level >= required_level
    }
    
    pub fn is_active(&self) -> bool {
        self.account_status == "active" &&
            (self.locked_until.is_none() ||
                self.locked_until.map_or(true, |locked| locked < chrono::Utc::now()))
    }
    
    pub fn is_ip_allowed(&self, ip: &str) -> bool {
        if let Some(Json::Array(whitelist)) = &self.ip_whitelist {
            if whitelist.is_empty() {
                return true;
            }
            whitelist.iter().any(|allowed_ip| {
                if let Json::String(ip_str) = allowed_ip {
                    ip_str == ip
                } else {
                    false
                }
            })
        } else {
            true
        }
    }
}