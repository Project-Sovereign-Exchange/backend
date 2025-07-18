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