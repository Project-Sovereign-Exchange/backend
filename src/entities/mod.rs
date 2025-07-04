pub mod users;
mod mfa_backup_codes;
mod known_login_locations;
mod login_history;
mod user_sessions;
mod address_history;

pub use users::Entity as Users;
pub use mfa_backup_codes::Entity as MfaBackupCodes;
pub use known_login_locations::Entity as KnownLoginLocations;
pub use login_history::Entity as LoginHistory;
pub use user_sessions::Entity as UserSessions;
pub use address_history::Entity as AddressHistory;

pub trait TimestampedUpdate {
    fn with_updated_timestamp() -> Self;
}