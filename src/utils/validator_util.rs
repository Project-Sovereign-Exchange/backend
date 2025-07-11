


pub struct ValidatorUtil;

impl ValidatorUtil {
    pub fn validate_email(email: &str) -> bool {
        let re = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        re.is_match(email)
    }

    pub fn validate_password(password: &str) -> bool {
        let re = regex::Regex::new(r"^(?=.*[A-Za-z])(?=.*\d)[A-Za-z\d]{10,}$").unwrap();
        re.is_match(password)
    }
}