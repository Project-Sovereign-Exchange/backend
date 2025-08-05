use std::collections::HashMap;
use actix_multipart::Multipart;
use serde::Deserialize;
use actix_web::{Responder, Result, web, post, get, delete};
use futures_util::TryStreamExt;
use uuid::Uuid;
use crate::app_state::AppState;
use crate::services::account::jwt_service::Claims;
use crate::services::marketplace::listing_service::ListingService;
use crate::services::marketplace::product_service::ProductService;

#[derive(Deserialize)]
pub struct CreateListingRequest {
    pub product_id: Uuid,
    pub price: i64,
    pub condition: String,
    pub quantity: i64,
    pub image_url: Option<String>,
    pub description: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateListingRequest {
    pub id: Uuid,
    pub price: Option<i64>,
    pub condition: Option<String>,
    pub quantity: Option<i64>,
    pub image_url: Option<String>,
    pub description: Option<String>,
}

#[post("/create")]
pub async fn create_listing(
    state: web::Data<AppState>,
    claims: Claims,
    request: web::Json<CreateListingRequest>,
) -> Result<impl Responder> {
    let request = request.into_inner();
    let listing_service = ListingService::new(state.as_ref().clone());

    match listing_service.create_listing(claims.sub, request).await {
        Ok(listing) => Ok(actix_web::HttpResponse::Ok().json(listing)),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),

    }
}

#[post("/update")]
pub async fn update_listing(
    state: web::Data<AppState>,
    claims: Claims,
    request: web::Json<UpdateListingRequest>,
) -> Result<impl Responder> {
    let request = request.into_inner();
    let listing_service = ListingService::new(state.as_ref().clone());

    match listing_service.update_listing(claims.sub, request).await {
        Ok(listing) => Ok(actix_web::HttpResponse::Ok().json(listing)),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}

#[get("/{id}")]
pub async fn get_listing(
    state: web::Data<AppState>,
    id: web::Path<uuid::Uuid>,
) -> Result<impl Responder> {
    let id = id.into_inner();
    let listing_service = ListingService::new(state.as_ref().clone());

    match listing_service.get_listing(id).await {
        Ok(listing) => Ok(actix_web::HttpResponse::Ok().json(listing)),
        Err(e) => Err(actix_web::error::ErrorInternalServerError("Fail to get listing by id")),
    }
}

#[delete("/{id}")]
pub async fn delete_listing(
    state: web::Data<AppState>,
    claims: Claims,
    id: web::Path<uuid::Uuid>,
) -> Result<impl Responder> {
    let id = id.into_inner();
    let listing_service = ListingService::new(state.as_ref().clone());

    match listing_service.delete_listing(claims.sub, id).await {
        Ok(true) => Ok(actix_web::HttpResponse::NoContent().finish()),
        Ok(false) => Err(actix_web::error::ErrorNotFound("Listing has already been deleted")),
        Err(_) => Err(actix_web::error::ErrorNotFound("Listing not found")),
    }
}