
use actix_web::web;
use serde::{Deserialize, Serialize};

pub mod account;
pub mod marketplace;
pub mod transactions;
pub mod admin;
pub mod integrations;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

pub fn configure_admin_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/api/v1/admin").configure(admin_routes));
}

pub fn configure_private_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/api/v1/private").configure(private_routes));
}

pub fn configure_public_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/api/v1/public").configure(public_routes));
}

fn admin_routes(cfg: &mut web::ServiceConfig) {}

fn private_routes(cfg: &mut web::ServiceConfig) {
        cfg.service(
            web::scope("/auth")
                .service(account::auth_handler::get_current_user)
                .service(account::auth_handler::logout)
        )
        .service(
            web::scope("/mfa")
                .service(account::mfa_handler::setup_mfa)
                .service(account::mfa_handler::enable_mfa)
                .service(account::mfa_handler::disable_mfa)
                .service(account::mfa_handler::verify_mfa),
        )
        .service(
            web::scope("/listing")
                .service(marketplace::listing_handler::create_listing)
                .service(marketplace::listing_handler::update_listing)
                .service(marketplace::listing_handler::delete_listing)
        )
        .service(
            web::scope("/product")
                .service(marketplace::product_handler::create_product)
                .service(marketplace::product_handler::delete_product)
                .service(marketplace::product_handler::create_product_variants)
                .service(marketplace::product_handler::upload_product_images)
        );
}

fn public_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/status").route(web::get().to(|| async { "OK" })))
        .service(
            web::scope("/auth")
                .service(account::auth_handler::login)
                .service(account::auth_handler::register),
        )
        .service(
            web::scope("/product")
                .service(marketplace::product_handler::get_product_by_id)
                .service(marketplace::product_handler::get_products)
                .service(marketplace::product_handler::get_product_variants)
                .service(marketplace::product_handler::get_number_of_products),
        )
        .service(
            web::scope("/listing")
                .service(marketplace::listing_handler::get_listing),
        )
        .service(
            web::scope("/games")
                .service(marketplace::game_handler::get_games)
                .service(marketplace::game_handler::get_sets_by_game),
        )
        .service(
            web::scope("/search")
                .service(integrations::meilisearch_handler::search_listings)
                .service(integrations::meilisearch_handler::search_products),
        );
}
