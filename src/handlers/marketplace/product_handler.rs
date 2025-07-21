use serde::Deserialize;
use actix_web::{delete, get, post, web, Responder, Result};
use crate::app_state::AppState;
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
    state: web::Data<AppState>,
    request: web::Json<CreateProductRequest>,
) -> Result<impl Responder> {
    let request = request.into_inner();
    let product_service = ProductService::new(state.as_ref().clone());
    
   match product_service.create_product(request).await {
       Ok(product) => Ok(web::Json(product)),
       Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
   }
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
    state: web::Data<AppState>,
    request: web::Json<UpdateProductRequest>,
) -> Result<impl Responder> {
    let request = request.into_inner();
    let product_service = ProductService::new(state.as_ref().clone());

    match product_service.update_product(request).await {
        Ok(product) => Ok(actix_web::HttpResponse::Ok().json(product)),
        Err(e) => {
            Err(actix_web::error::ErrorInternalServerError("Product failed to update"))
        },
    }
}

#[delete("/{id}")]
pub async fn delete_product(
    state: web::Data<AppState>,
    id: web::Path<uuid::Uuid>,
) -> Result<impl Responder> {
    let id = id.into_inner();
    let product_service = ProductService::new(state.as_ref().clone());
    
    match product_service.delete_product(id).await {
        Ok(true) => Ok(actix_web::HttpResponse::NoContent().finish()),
        Ok(false) => Err(actix_web::error::ErrorNotFound("Product not found")),
        Err(e) => {
            Err(actix_web::error::ErrorInternalServerError("Product failed to delete"))
        }
    }
}

#[get("/{id}")]
pub async fn get_product(
    state: web::Data<AppState>,
    id: web::Path<uuid::Uuid>,
) -> Result<impl Responder> {
    let id = id.into_inner();
    let product_service = ProductService::new(state.as_ref().clone());
    
    match product_service.get_product_by_id(&id).await {
        Ok(product) => Ok(actix_web::HttpResponse::Ok().json(product)),
        Err(e) => {
            Err(actix_web::error::ErrorInternalServerError("Product failed to query"))
        }
    }
}

