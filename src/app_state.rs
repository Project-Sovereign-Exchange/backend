use std::env;
use std::sync::Arc;
use std::time::Duration;
use reqwest::Client;
use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use tracing::log;
use crate::config::config::Config;
use crate::services::integrations::meilisearch_service::MeilisearchService;
use crate::services::integrations::r2_service::R2Client;
use crate::services::integrations::stripe_service::StripeClient;

#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub stripe_client: Arc<StripeClient>,
    pub meilisearch_client: Arc<meilisearch_sdk::client::Client>,
    pub r2_client: Arc<R2Client>
}

impl AppState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let config = Config::get();
        
        let mut db_options = ConnectOptions::new(config.database_url.clone());
        db_options.max_connections(config.max_db_connections)
            .min_connections(5)
            .connect_timeout(Duration::from_secs(8))
            .idle_timeout(Duration::from_secs(8))
            .acquire_timeout(Duration::from_secs(8))
            .max_lifetime(Duration::from_secs(8))
            .sqlx_logging(true)
            .sqlx_logging_level(log::LevelFilter::Info);
        
        let db = Database::connect(db_options).await?;
        
        let stripe_client = Arc::new(StripeClient::new());

        let meilisearch_client = Arc::new(
            meilisearch_sdk::client::Client::new("http://localhost:7700", Some("your-master-key"))
                .unwrap_or_else(|e| panic!("Failed to initialize client: {}", e))
        );

        let r2_client = Arc::new(
            R2Client::new(
                &config.r2_account_id,
                &config.r2_access_key_id,
                &config.r2_secret_access_key,
                &config.r2_custom_domain,
            ).await?
        );
        
        Ok(Self {
            db,
            stripe_client,
            meilisearch_client,
            r2_client,
        })
    }
}