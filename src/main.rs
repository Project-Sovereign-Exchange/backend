use actix_web::dev::Service;
use actix_web::{web, App, HttpServer, Result};
use actix_web::web::Data;
use sea_orm::ColumnType::Uuid;
use crate::app_state::AppState;
use crate::services::integrations::meilisearch_service::MeilisearchService;
use crate::utils::cli_util::CliUtil;
use crate::utils::message_util::MessageUtil;

mod handlers;
mod services;
mod entities;
mod config;
mod utils;
mod middleware;
mod database;
mod app_state;

#[actix_web::main]
async fn main() -> Result<()> {
    
    CliUtil::print_logo();
    MessageUtil::info("Starting TCGEmporium server...");

    let config = config::config::Config::get();
    let app_state = match AppState::new().await {
        Ok(state) => {
            MessageUtil::info("Application state initialized successfully");
            state
        }
        Err(e) => {
            MessageUtil::error(&format!("Failed to initialize application state: {}", e));
            std::process::exit(1);
        }
    };

    let meilisearch_service = MeilisearchService::new(app_state.clone());
    match meilisearch_service.validate_indexes().await {
        Ok(_) => MessageUtil::info("Meilisearch indexes validated successfully"),
        Err(e) => {
            MessageUtil::error(&format!("Failed to validate Meilisearch indexes: {}", e));
            std::process::exit(1);
        }
    }

    let server = HttpServer::new(move || {
        App::new()
            .app_data(Data::new(app_state.clone()))
            .wrap(config::cors::configure_cors())
            .wrap_fn(|req, srv| {
                let start_time = std::time::Instant::now();
                let request_id = uuid::Uuid::new_v4();
                
                middleware::logger_middleware::LoggerMiddleware::log_request(&req, req.path(), &request_id);
                let fut = srv.call(req);
                async move {
                    let res = fut.await?;
                    middleware::logger_middleware::LoggerMiddleware::log_response(res.status().as_u16(), &request_id, Some(start_time.elapsed().as_millis() as u64));
                    Ok(res)
                }
            })
            .configure(handlers::configure_public_routes)
            
            .service(
                web::scope("")
                    .wrap(middleware::auth_middleware::AuthMiddleware)
                    .configure(handlers::configure_private_routes)
            )
    })
        .bind(format!("{}:{}", config.host, config.port))?
        .run()
        .await;
    
    match server {
        Ok(_) => {
            MessageUtil::info("Server started successfully.");
            Ok(())
        },
        Err(e) => {
            MessageUtil::error(&format!("Failed to start server: {}", e));
            Err(actix_web::error::ErrorInternalServerError("Server startup failed"))
        }
    }
}
