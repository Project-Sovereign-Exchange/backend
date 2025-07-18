use serde::Deserialize;
use actix_web::{delete, get, post, web, Responder, Result};
use crate::services::marketplace::product_service::ProductService;

#[derive(Deserialize)]
pub struct CreateProductRequest {
    pub name: String,
    pub image_url: Option<String>,
    pub game: String,
    pub expansion: Option<String>,
    pub metadata: serde_json::Value,
}

#[post("/create")]
pub async fn create_product(
    request: web::Json<CreateProductRequest>,
) -> Result<impl Responder> {
    let request = request.into_inner();
    
    let product = ProductService::create_product(request)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e))?;

    Ok(actix_web::HttpResponse::Created()
        .json(product))
}

#[derive(Deserialize)]
pub struct UpdateProductRequest {
    pub id: uuid::Uuid,
    pub name: Option<String>,
    pub image_url: Option<String>,
    pub game: Option<String>,
    pub expansion: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[post("/update")]
pub async fn update_product(
    request: web::Json<UpdateProductRequest>,
) -> Result<impl Responder> {
    let request = request.into_inner();
    
    let updated_product = ProductService::update_product(request)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e))?;

    Ok(actix_web::HttpResponse::Ok()
        .json(updated_product))
}

#[delete("/{id}")]
pub async fn delete_product(
    id: web::Path<uuid::Uuid>,
) -> Result<impl Responder> {
    let id = id.into_inner();
    
    let deleted = ProductService::delete_product(id)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e))?;

    if deleted {
        Ok(actix_web::HttpResponse::NoContent().finish())
    } else {
        Err(actix_web::error::ErrorNotFound("Product not found"))
    }
}

#[get("/{id}")]
pub async fn get_product(
    id: web::Path<uuid::Uuid>,
) -> Result<impl Responder> {
    let id = id.into_inner();
    
    let product = ProductService::get_product_by_id(&id)
        .await
        .map_err(|e| actix_web::error::ErrorBadRequest(e))?
        .ok_or_else(|| actix_web::error::ErrorNotFound("Product not found"))?;

    Ok(actix_web::HttpResponse::Ok()
        .json(product))
}

