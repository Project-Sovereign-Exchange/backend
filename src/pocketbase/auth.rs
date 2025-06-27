use serde::{Deserialize, Serialize};

pub struct  PocketBaseClient {
    base_url: String,
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub token: String,
    pub record: Record,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Record {
    pub id: String,
    #[serde(rename = "collectionId")]
    pub collection_id: String,
    #[serde(rename = "collectionName")]
    pub collection_name: String,
    pub created: String,
    pub updated: String,
    pub name: String,
    pub email: String,
    #[serde(rename = "emailVisibility")]
    pub email_visibility: bool,
    pub verified: bool,
    pub avatar: String,
}

impl PocketBaseClient {
    pub fn new(base_url: &str, token: &str) -> Self {
        PocketBaseClient {
            base_url: base_url.to_string(),
            token: token.to_string(),
        }
    }

    pub fn get_base_url(&self) -> &str {
        &self.base_url
    }

    pub fn get_token(&self) -> &str {
        &self.token
    }

    pub async fn create_user(&self, username: &str, email: &str, password: &str) -> Result<User, String> {
        if username.is_empty() || email.is_empty() || password.is_empty() {
            return Err("Username, email or password cannot be empty".to_string());
        }

        let url = format!("{}/api/collections/users/records", self.base_url);

        let client = reqwest::Client::new();
        let response = client
            .post(url)
            .json(&serde_json::json!({
                "email": email,
                "emailVisibility": true,
                "password": password,
                "passwordConfirm": password,
                "name": username
            }))
            .send()
            .await
            .expect("Request failed");

        if !response.status().is_success() {
            return Err(format!(
                "Failed to create user: {}",
                response.status()
            ));
        }

        Self::auth_with_password(&self, email, password).await
            .map_err(|e| format!("Failed to authenticate after creation: {}", e))
    }

    async fn auth_with_password(&self, email: &str, password: &str) -> Result<User, String> {
        if email.is_empty() || password.is_empty() {
            return Err("Username or password cannot be empty".to_string());
        }

        let url = format!("{}/api/collections/users/auth-with-password", self.base_url);

        let client = reqwest::Client::new();
        let response = client
            .post(url)
            .json(&serde_json::json!({
                "identity": email,
                "password": password,
            }))
            .send()
            .await
            .expect("Request failed");

        if !response.status().is_success() {
            return Err(format!(
                "Failed to authenticate: {}",
                response.status()
            ));
        }

        let text = response.text().await.expect("Request failed");
        println!("{}", text);

        /*
        response.json::<User>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))

         */
        let user: User = serde_json::from_str(&text)
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(user)
    }

    pub async fn auth_with_oauth2(&self, provider: &str, code: &str, verifier: &str, redirect_url: &str) -> Result<User, String> {
        if provider.is_empty() || code.is_empty() || verifier.is_empty() || redirect_url.is_empty() {
            return Err("Provider, code, verifier or redirect_url cannot be empty".to_string());
        }

        let url = format!("{}/api/collections/users/auth-with-oauth2", self.base_url);

        let client = reqwest::Client::new();
        let response = client
            .post(url)
            .json(&serde_json::json!({
                "provider": provider,
                "code": code,
                "codeVerifier": verifier,
                "redirectUrl": redirect_url,
            }))
            .send()
            .await
            .expect("Request failed");

        if !response.status().is_success() {
            return Err(format!(
                "Failed to authenticate with OAuth2: {}",
                response.status()
            ));
        }

        let text = response.text().await.expect("Request failed");
        let user: User = serde_json::from_str(&text)
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(user)
    }

    async fn get_user(&self, user_id: &str) -> Result<User, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let response = client
            .get(&format!(
                "{}/api/collections/users/records/{}",
                self.base_url, user_id
            ))
            .header("Authorization", format!("Bearer {}", self.token))
            .send()
            .await?;

        if response.status().is_success() {
            let user: User = response.json().await?;
            Ok(user)
        } else {
            Err("User not found".into())
        }
    }
    
    async fn check_username_exists(&self, username: &str) -> Result<bool, String> {
        if username.is_empty() {
            return Err("Username cannot be empty".to_string());
        }

        let url = format!("{}/api/collections/users/records/{}", self.base_url, username);

        let client = reqwest::Client::new();
        let response = client
            .get(url)
            .send()
            .await
            .expect("Request failed");

        if response.status().is_success() {
            Ok(true)
        } else if response.status().as_u16() == 404 {
            Ok(false)
        } else {
            Err(format!("Failed to check username existence: {}", response.status()))
        }
    }
    
    async fn check_email_exists(&self, email: &str) -> Result<bool, String> {
        if email.is_empty() {
            return Err("Email cannot be empty".to_string());
        }

        let url = format!("{}/api/collections/users/records/{}", self.base_url, email);

        let client = reqwest::Client::new();
        let response = client
            .get(url)
            .send()
            .await
            .expect("Request failed");

        if response.status().is_success() {
            Ok(true)
        } else if response.status().as_u16() == 404 {
            Ok(false)
        } else {
            Err(format!("Failed to check email existence: {}", response.status()))
        }
    }
}