use std::collections::HashMap;
use actix_multipart::Multipart;
use serde::{Deserialize, Serialize};
use actix_web::{delete, get, post, web, HttpResponse, Responder, Result};
use chrono::{DateTime, Utc};
use futures_util::TryStreamExt;
use uuid::Uuid;
use validator::Validate;
use crate::app_state::AppState;
use crate::entities::{product_variants, products};
use crate::handlers::ApiResponse;
use crate::services::account::jwt_service::Claims;
use crate::services::marketplace::product_service::ProductService;

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateProductRequest {
    #[validate(length(min = 1, max = 255, message = "Name must be between 1 and 255 characters"))]
    pub name: String,
    #[validate(length(max = 2000, message = "Description cannot exceed 2000 characters"))]
    pub description: Option<String>,
    #[validate(length(min = 1, max = 100, message = "Category is required"))]
    pub category: String,
    #[validate(length(max = 100, message = "Subcategory cannot exceed 100 characters"))]
    pub subcategory: Option<String>,
    #[validate(length(min = 1, max = 100, message = "Game is required"))]
    pub game: String,
    #[validate(length(max = 100))]
    pub set: Option<String>,
    #[validate(url(message = "Must be a valid URL"))]
    pub base_image_url: Option<String>,
    #[validate(length(min = 1, message = "At least one variant is required"))]
    pub variants: Vec<CreateVariantRequest>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize, Validate)]
pub struct CreateVariantRequest {
    #[validate(length(min = 1, max = 100, message = "Variant name is required"))]
    pub name: String,

    #[validate(length(max = 50))]
    pub set_number: Option<String>,

    pub is_primary: bool,

    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ProductImageUploadRequest {
    pub uploads: Vec<ImageUploadRequest>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ImageUploadRequest {
    pub variant_id: i32,
    pub variant_name: String,
    pub front_image: Option<ImageData>,
    pub back_image: Option<ImageData>,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
    pub file_data: Option<Vec<u8>>,
}

#[derive(Debug, Serialize)]
pub struct ProductResponse {
    pub product: products::Model,
    pub variants: Vec<product_variants::Model>,
}

#[derive(Debug, Serialize)]
pub struct VariantResponse {
    pub id: i32,
    pub variant_name: String,
    pub set_number: Option<String>,
    pub is_primary: bool,
    pub has_back_image: bool,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
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
            serde_json::json!({
                "success": true,
                "message": "Product created successfully",
                "data": product,
            })
        )),
        Err(e) => Err(actix_web::error::ErrorInternalServerError(e)),
    }
}

#[post("/{product_id}/variants")]
pub async fn create_product_variants(
    state: web::Data<AppState>,
    path: web::Path<(Uuid)>,
    request: web::Json<Vec<CreateVariantRequest>>,
) -> Result<impl Responder> {
    let product_id = path.into_inner();
    let request = request.into_inner();

    let product_service = ProductService::new(state.as_ref().clone());

    match product_service.create_product_variants(product_id, request).await {
        Ok(variants) => Ok(HttpResponse::Ok().json(
            serde_json::json!({
                "success": true,
                "message": "Variants created successfully",
                "variants": variants,
            })
        )),
        Err(e) => Ok(HttpResponse::InternalServerError().json(format!("Error: {}", e))),
    }
}

#[get("/{product_id}/variants")]
pub async fn get_product_variants(
    state: web::Data<AppState>,
    product_id: web::Path<uuid::Uuid>,
) -> Result<impl Responder> {
    let product_id = product_id.into_inner();
    let product_service = ProductService::new(state.as_ref().clone());

    match product_service.get_product_variants(&product_id).await {
        Ok(variants) => {
            Ok(HttpResponse::Ok().json(ApiResponse {
                success: true,
                message: "Product variants retrieved successfully".to_string(),
                data: Some(variants),
            }))
        },
        Err(e) => {
            Err(actix_web::error::ErrorInternalServerError("Failed to get product variants"))
        }
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

    let upload_request = match parse_image_upload_form(payload).await {
        Ok(request) => request,
        Err(e) => return Ok(HttpResponse::BadRequest().json(
            serde_json::json!({
                "success": false,
                "message": format!("Failed to parse upload form: {}", e),
            })
        )),
    };

    let product_service = ProductService::new(state.as_ref().clone());

    match product_service.upload_product_images(&claims.sub, &product_id, upload_request).await {
        Ok(_) => Ok(HttpResponse::Ok().json(
            serde_json::json!({
                "success": true,
                "message": "Images uploaded successfully",
            })
        )),
        Err(e) => Ok(HttpResponse::InternalServerError().json(
            serde_json::json!({
                "success": false,
                "message": format!("Upload failed: {}", e),
            })
        )),
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
        .map_err(|e| {
            println!("Failed to get total number of products: {}", e);
            actix_web::error::ErrorInternalServerError("Failed to get total number of products")
        })?;

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

#[get("/{id}")]
pub async fn get_product_by_id(
    state: web::Data<AppState>,
    path: web::Path<Uuid>,
) -> Result<impl Responder> {
    let product_id = path.into_inner();
    let product_service = ProductService::new(state.as_ref().clone());

    match product_service.get_product_by_id(&product_id).await {
        Ok(Some(product)) => {
            Ok(HttpResponse::Ok().json(ApiResponse {
                success: true,
                message: "Product retrieved successfully".to_string(),
                data: Some(product),
            }))
        },
        Ok(None) => {
            Err(actix_web::error::ErrorNotFound("Product not found"))
        },
        Err(e) => {
            println!("Failed to get product by ID: {}", e);
            Err(actix_web::error::ErrorInternalServerError("Failed to get product"))
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

            let variant_id: i32 = parts[1].parse()
                .map_err(|_| format!("Invalid variant_id: {}", parts[1]))?;
            let field_type = parts[2];

            let upload = uploads.entry(variant_id.to_string()).or_insert(ImageUploadRequest {
                variant_id,
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
                _ => {}
            }
        }
    }

    Ok(ProductImageUploadRequest {
        uploads: uploads.into_values().collect(),
    })
}


