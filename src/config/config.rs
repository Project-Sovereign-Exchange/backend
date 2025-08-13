use std::env;
use std::sync::OnceLock;
use dotenvy::dotenv;
use crate::utils::message_util::MessageUtil;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub max_db_connections: u32,
    pub host: String,
    pub port: u16,
    pub stripe_key: String,
    pub meilisearch_url: String,
    pub meilisearch_key: String,
    pub r2_account_id: String,
    pub r2_access_key_id: String,
    pub r2_secret_access_key: String,
    pub r2_custom_domain: String,
}

static CONFIG: OnceLock<Config> = OnceLock::new();

impl Config {
    fn from_env() -> Result<Self, ()> {
        MessageUtil::info("Loading configuration from environment variables...");

        dotenv().ok();

        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .map_err(|e| {
                    MessageUtil::error(&format!("DATABASE_URL must be set: {}", e));
                    ()
                })?,
            jwt_secret: env::var("JWT_SECRET")
                .map_err(|e| {
                    MessageUtil::error(&format!("JWT_SECRET must be set: {}", e));
                    ()
                })?,
            max_db_connections: env::var("MAX_DB_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .map_err(|e| {
                    MessageUtil::error(&format!("MAX_DB_CONNECTIONS must be a valid number: {}", e));
                    ()
                })?,
            host: env::var("HOST")
                .unwrap_or_else(|_| "localhost".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .map_err(|e| {
                    MessageUtil::error(&format!("PORT must be a valid number: {}", e));
                    ()
                })?,
            stripe_key: env::var("STRIPE_KEY")
                .map_err(|e| {
                    MessageUtil::error(&format!("STRIPE_KEY must be set: {}", e));
                    ()
                })?,
            meilisearch_url: env::var("MEILISEARCH_URL")
                .map_err(|e| {
                    MessageUtil::error(&format!("MEILISEARCH_URL must be set: {}", e));
                    ()
                })?,
            meilisearch_key: env::var("MEILISEARCH_KEY")
                .map_err(|e| {
                    MessageUtil::error(&format!("MEILISEARCH_KEY must be set: {}", e));
                    ()
                })?,
            r2_account_id: env::var("R2_ACCOUNT_ID")
                .map_err(|e| {
                    MessageUtil::error(&format!("R2_ACCOUNT_ID must be set: {}", e));
                    ()
                })?,
            r2_access_key_id: env::var("R2_ACCESS_KEY_ID")
                .map_err(|e| {
                    MessageUtil::error(&format!("R2_ACCESS_KEY_ID must be set: {}", e));
                    ()
                })?,
            r2_secret_access_key: env::var("R2_SECRET_ACCESS_KEY")
                .map_err(|e| {
                    MessageUtil::error(&format!("R2_SECRET_ACCESS_KEY must be set: {}", e));
                    ()
                })?,
            r2_custom_domain: env::var("R2_CUSTOM_DOMAIN")
                .map_err(|e| {
                    MessageUtil::error(&format!("R2_CUSTOM_DOMAIN must be set: {}", e));
                    ()
                })?,
        })
    }
    
    pub fn get() -> &'static Self {
        CONFIG.get_or_init(|| {
            Config::from_env().unwrap_or_else({
                |_| {
                    MessageUtil::error("Failed to load configuration from environment variables");
                    panic!();
                }
            })
        })
    }
}