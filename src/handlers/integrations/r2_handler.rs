/*
use actix_web::{post, web, HttpResponse, Result};
use serde::Deserialize;
use validator::Validate;
use crate::app_state::AppState;

#[derive(Debug, Deserialize, Validate)]
pub struct GetProductUploadUrlRequest {
    #[validate(regex = "^(jpg|jpeg|png|webp)$")]
    pub file_extension: String,

    #[validate(regex = "^image/(jpeg|png|webp)$")]
    pub content_type: String,

    #[validate(range(min = 1, max = 10485760))]
    pub file_size: u64,

    #[validate(regex = "^(cards|accessories|sealed)$")]
    pub category: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct GetListingUploadUrlRequest {
    #[validate(regex = "^(jpg|jpeg|png|webp)$")]
    pub file_extension: String,

    #[validate(regex = "^image/(jpeg|png|webp)$")]
    pub content_type: String,

    #[validate(range(min = 1, max = 10485760))]
    pub file_size: u64,

    #[validate(regex = "^(cards|accessories|sealed)$")]
    pub category: String,
}

#[post("/upload/get-product-url")]
pub async fn get_upload_url(
    req: web::Json<GetProductUploadUrlRequest>,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {

}

#[post("/upload/get-listing-url")]
pub async fn get_listing_upload_url(
    req: web::Json<GetUploadUrlRequest>,
    state: web::Data<AppState>,
) -> Result<HttpResponse> {

}
 */


