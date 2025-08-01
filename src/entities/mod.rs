pub mod users;
pub mod mfa_backup_codes;
pub mod known_login_locations;
pub mod login_history;
pub mod user_sessions;
pub mod address_history;
pub mod token_blacklist;
pub mod products;
pub mod listings;
pub mod admin_users;

pub use users::Entity as Users;
pub use mfa_backup_codes::Entity as MfaBackupCodes;
pub use known_login_locations::Entity as KnownLoginLocations;
pub use login_history::Entity as LoginHistory;
pub use user_sessions::Entity as UserSessions;
pub use address_history::Entity as AddressHistory;

pub trait TimestampedUpdate {
    fn with_updated_timestamp() -> Self;
}