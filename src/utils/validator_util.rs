use once_cell::sync::Lazy;
use regex::Regex;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ValidationError {
    #[error("Invalid email format")]
    InvalidEmail,
    #[error("Password must be at least 10 characters long")]
    PasswordTooShort,
    #[error("Password must contain at least one letter")]
    PasswordMissingLetter,
    #[error("Password must contain at least one number")]
    PasswordMissingNumber,
    #[error("Password contains invalid characters")]
    PasswordInvalidCharacters,
    #[error("Email is too long (max 254 characters)")]
    EmailTooLong,
    #[error("Email is required")]
    EmailRequired,
    #[error("Password is required")]
    PasswordRequired,
}

pub type ValidationResult<T> = Result<T, ValidationError>;

static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
        .expect("Invalid email regex")
});

static PASSWORD_LETTER_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[A-Za-z]").expect("Invalid password letter regex")
});

static PASSWORD_DIGIT_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"\d").expect("Invalid password digit regex")
});

static PASSWORD_VALID_CHARS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^[A-Za-z\d@$!%*?&]+$").expect("Invalid password chars regex")
});

pub struct ValidatorUtil;

impl ValidatorUtil {

    pub fn validate_email(email: &str) -> ValidationResult<()> {
        if email.is_empty() {
            return Err(ValidationError::EmailRequired);
        }

        if email.len() > 254 {
            return Err(ValidationError::EmailTooLong);
        }

        if !EMAIL_REGEX.is_match(email) {
            return Err(ValidationError::InvalidEmail);
        }

        Ok(())
    }

    pub fn validate_password(password: &str) -> ValidationResult<()> {
        if password.is_empty() {
            return Err(ValidationError::PasswordRequired);
        }

        if password.len() < 10 {
            return Err(ValidationError::PasswordTooShort);
        }

        if !PASSWORD_LETTER_REGEX.is_match(password) {
            return Err(ValidationError::PasswordMissingLetter);
        }

        if !PASSWORD_DIGIT_REGEX.is_match(password) {
            return Err(ValidationError::PasswordMissingNumber);
        }

        if !PASSWORD_VALID_CHARS_REGEX.is_match(password) {
            return Err(ValidationError::PasswordInvalidCharacters);
        }

        Ok(())
    }
}