use serde::Deserialize;
use actix_web::{Responder, Result, web, post};
use sea_orm::prelude::Decimal;
use crate::services::account::jwt_service::Claims;
use crate::services::marketplace::listing_service::ListingService;

#[derive(Deserialize)]
pub struct CreateListingRequest {
    pub product_id: uuid::Uuid,
    pub price: Decimal,
    pub condition: String,
    pub quantity: i32,
    pub image_url: Option<String>,
    pub description: Option<String>,
}

#[post("/create")]
pub async fn create_listing(
    claims: Claims,
    request: web::Json<CreateListingRequest>,
) -> Result<impl Responder> {
    let request = request.into_inner();

    let listing = ListingService::create_listing(claims.sub, request)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e))?;

    Ok(actix_web::HttpResponse::Created()
        .json(listing))
}