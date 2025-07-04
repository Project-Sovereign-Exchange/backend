use std::env;
use crate::utils::message_util::MessageUtil;

#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub server_address: String,
    pub server_port: String,
    pub jwt_secret: String,
    pub max_db_connections: u32,
}

impl Config {
    pub fn from_env() -> Result<Self, ()> {
        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .map_err(|e| {
                    MessageUtil::error(&format!("DATABASE_URL must be set: {}", e));
                    ()
                })?,
            server_address: env::var("SERVER_ADDRESS")
                .map_err(|e| {
                    MessageUtil::error(&format!("SERVER_ADDRESS must be set: {}", e));
                    ()
                })?,
            server_port: env::var("SERVER_PORT")
                .map_err(|e| {
                    MessageUtil::error(&format!("SERVER_PORT must be set: {}", e));
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
        })
    }
}