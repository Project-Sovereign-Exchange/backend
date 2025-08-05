use std::collections::HashMap;
use actix_multipart::Multipart;
use serde::{Deserialize, Serialize};
use actix_web::{delete, get, post, web, HttpResponse, Responder, Result};
use futures_util::TryStreamExt;
use uuid::Uuid;
use crate::app_state::AppState;
use crate::entities::products;
use crate::services::account::jwt_service::Claims;
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

#[derive(Debug, Clone)]
pub struct ImageUploadRequest {
    pub variant_id: String,
    pub variant_name: String,
    pub front_image: Option<ImageData>,
    pub back_image: Option<ImageData>,
    pub is_primary: bool,
}

#[derive(Debug, Clone)]
pub struct ImageData {
    pub file_data: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct ProductImageUploadRequest {
    pub uploads: Vec<ImageUploadRequest>,
}

#[derive(Deserialize, Serialize)]
pub struct ProductResponse {
    success: bool,
    message: String,
    product: products::Model,
}

#[post("")]
pub async fn create_product(
    state: web::Data<AppState>,
    request: web::Json<CreateProductRequest>,
) -> Result<impl Responder> {
    let request = request.into_inner();
    let product_service = ProductService::new(state.as_ref().clone());
    
   match product_service.create_product(request).await {
       Ok(product) => Ok(HttpResponse::Ok().json(
           ProductResponse {
               success: true,
               message: "Product created successfully".to_string(),
               product,
           }
       )),
       Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
   }
}

#[post("/{product_id}/images")]
pub async fn upload_product_images(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
    claims: Claims,
    payload: Multipart,
) -> Result<impl Responder> {
    let product_id = path.into_inner();

    match parse_image_upload_form(payload).await {
        Ok(upload_request) => {
            let product_service = ProductService::new(state.as_ref().clone());

            match product_service.upload_product_images(claims.sub, product_id, upload_request).await {
                Ok(updated_product) => Ok(HttpResponse::Ok().json(
                    ProductResponse {
                        success: true,
                        message: "Images uploaded successfully".to_string(),
                        product: updated_product,
                    }
                )),
                Err(e) => Ok(HttpResponse::InternalServerError().json(format!("Error: {}", e))),
            }
        }
        Err(e) => Ok(HttpResponse::BadRequest().json(format!("Invalid form data: {}", e))),
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
        Ok(product) => Ok(HttpResponse::Ok().json(product)),
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

async fn parse_image_upload_form(mut payload: Multipart) -> Result<ProductImageUploadRequest, String> {
    let mut uploads: HashMap<String, ImageUploadRequest> = HashMap::new();

    while let Some(mut field) = payload.try_next().await.map_err(|e| e.to_string())? {
        let field_name = field.name().unwrap_or("").to_string();

        if field_name.starts_with("variant_") {
            let parts: Vec<&str> = field_name.split('_').collect();
            if parts.len() < 3 {
                continue;
            }

            let variant_id = parts[1].to_string();
            let field_type = parts[2];

            let upload = uploads.entry(variant_id.clone()).or_insert(ImageUploadRequest {
                variant_id: variant_id.clone(),
                variant_name: format!("Variant {}", variant_id),
                front_image: None,
                back_image: None,
                is_primary: false,
            });

            match field_type {
                "name" => {
                    let mut field_data = Vec::new();
                    while let Some(chunk) = field.try_next().await.map_err(|e| e.to_string())? {
                        field_data.extend_from_slice(&chunk);
                    }
                    upload.variant_name = String::from_utf8(field_data).map_err(|e| e.to_string())?;
                }
                "primary" => {
                    let mut field_data = Vec::new();
                    while let Some(chunk) = field.try_next().await.map_err(|e| e.to_string())? {
                        field_data.extend_from_slice(&chunk);
                    }
                    let value = String::from_utf8(field_data).map_err(|e| e.to_string())?;
                    upload.is_primary = value == "true";
                }
                "front" => {
                    if field.content_type().is_some() {
                        let mut file_data = Vec::new();
                        while let Some(chunk) = field.try_next().await.map_err(|e| e.to_string())? {
                            file_data.extend_from_slice(&chunk);
                        }
                        upload.front_image = Some(ImageData {
                            file_data: Some(file_data),
                        });
                    }
                }
                "front_url" => {
                    let mut field_data = Vec::new();
                    while let Some(chunk) = field.try_next().await.map_err(|e| e.to_string())? {
                        field_data.extend_from_slice(&chunk);
                    }
                    let url = String::from_utf8(field_data).map_err(|e| e.to_string())?;
                    if !url.is_empty() {
                        upload.front_image = Some(ImageData {
                            file_data: None,
                        });
                    }
                }
                "back" => {
                    if field.content_type().is_some() {
                        let mut file_data = Vec::new();
                        while let Some(chunk) = field.try_next().await.map_err(|e| e.to_string())? {
                            file_data.extend_from_slice(&chunk);
                        }
                        upload.back_image = Some(ImageData {
                            file_data: Some(file_data),
                        });
                    }
                }
                "back_url" => {
                    let mut field_data = Vec::new();
                    while let Some(chunk) = field.try_next().await.map_err(|e| e.to_string())? {
                        field_data.extend_from_slice(&chunk);
                    }
                    let url = String::from_utf8(field_data).map_err(|e| e.to_string())?;
                    if !url.is_empty() {
                        upload.back_image = Some(ImageData {
                            file_data: None,
                        });
                    }
                }
                _ => {}
            }
        }
    }

    Ok(ProductImageUploadRequest {
        uploads: uploads.into_values().collect(),
    })
}


