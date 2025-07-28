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
    pub set_number: Option<String>,
    pub category: String,
    pub subcategory: Option<String>,
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

#[derive(Deserialize)]
pub struct ProductQuery {
    pub offset: Option<u64>,
    pub limit: Option<u64>,
}

#[get("")]
pub async fn get_products(
    state: web::Data<AppState>,
    query: web::Query<ProductQuery>,
) -> Result<impl Responder> {
    let offset = query.offset.unwrap_or(0);
    let limit = query.limit.unwrap_or(10);
    let product_service = ProductService::new(state.as_ref().clone());

    let total_number = product_service.get_number_of_products()
        .await
        .map_err(|_| actix_web::error::ErrorInternalServerError("Failed to get total number of products"))?;

    if limit == 0 {
        return Err(actix_web::error::ErrorBadRequest("Limit must be greater than 0"));
    }

    if offset > total_number {
        return Err(actix_web::error::ErrorBadRequest("Offset exceeds total number of products"));
    }

    match product_service.get_products(offset, limit).await {
        Ok(products) => {
            Ok(actix_web::HttpResponse::Ok().json(
                serde_json::json!({
                    "products": products,
                    "offset": offset,
                    "limit": limit,
                    "total": total_number,
                    "more": offset + limit < total_number
                })
            ))},
        Err(e) => {
            println!("Failed to get products: {}", e);
            Err(actix_web::error::ErrorInternalServerError("Products failed to query"))
        }
    }
}

#[get("/count")]
pub async fn get_number_of_products(
    state: web::Data<AppState>,
) -> Result<impl Responder> {
    let product_service = ProductService::new(state.as_ref().clone());

    match product_service.get_number_of_products().await {
        Ok(count) => Ok(actix_web::HttpResponse::Ok().json(count)),
        Err(e) => {
            Err(actix_web::error::ErrorInternalServerError("Failed to count products"))
        }
    }
}


