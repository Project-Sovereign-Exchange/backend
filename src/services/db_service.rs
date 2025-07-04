use std::env;
use sea_orm::{Database, DatabaseConnection};
use tokio::sync::OnceCell;

static DB: OnceCell<DatabaseConnection> = OnceCell::const_new();

pub struct DbService {}

impl DbService {
    pub async fn init() -> Result<(), anyhow::Error> {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgresql://user:password@localhost/dbname".to_string());

        let db = Database::connect(&database_url).await?;
        
        DB.set(db).map_err(|_| anyhow::anyhow!("Database already initialized"))?;

        println!("Database connection initialized successfully");
        Ok(())
    }
    
    pub async fn get() -> Result<&'static DatabaseConnection, anyhow::Error> {
        DB.get().ok_or_else(|| anyhow::anyhow!("Database connection not initialized"))
    }
}