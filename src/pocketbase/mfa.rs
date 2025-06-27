use serde::{Deserialize, Serialize};
use crate::pocketbase::auth::PocketBaseClient;

struct MfaService {
    pocket_base_client: PocketBaseClient,
}

#[derive(Serialize, Deserialize)]
pub struct MfaSetupRequest {
    pub user_id: String,
    pub issuer: String,
    pub account_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct MfaSetupResponse {
    pub secret: String,
    pub qr_code: String,
    pub backup_codes: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct MfaVerifyRequest {
    pub user_id: String,
    pub token: String,
    pub is_backup_code: Option<bool>,
}

impl MfaService {
    pub fn new(pocket_base_client: PocketBaseClient) -> Self {
        MfaService { pocket_base_client }
    }

    pub async fn enable_mfa(&self, user_id: &str) -> Result<(), String> {
        let url = format!("{}/api/collections/users/records/{}/enable-mfa", self.pocket_base_client.get_base_url(), user_id);
        
        let client = reqwest::Client::new();
        let response = client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.pocket_base_client.get_token()))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            return Err(format!("Failed to enable MFA: {}", response.status()));
        }

        Ok(())
    }
}