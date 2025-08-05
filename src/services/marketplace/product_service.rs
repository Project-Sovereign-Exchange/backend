use std::collections::HashMap;
use sea_orm::{ActiveModelTrait, EntityTrait, QuerySelect, Set, PaginatorTrait, JsonValue};
use serde::{Deserialize, Serialize};
use crate::entities::products;
use crate::handlers::marketplace::product_handler::{CreateProductRequest, ProductImageUploadRequest, UpdateProductRequest};
use uuid::Uuid;
use crate::app_state::AppState;
use crate::entities::products::string_to_product_category;
use crate::services::integrations::meilisearch_service::MeilisearchService;
use crate::services::integrations::r2_service::R2Service;

pub struct ProductService {
    pub state: AppState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageVariantData {
    pub front: Option<String>,
    pub back: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<HashMap<String, ImageVariantData>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub variants: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_variant: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant_names: Option<HashMap<String, String>>,

    #[serde(flatten)]
    pub additional: HashMap<String, JsonValue>,
}


impl ProductService {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
    
    pub async fn create_product(
        &self,
        request: CreateProductRequest,
    ) -> Result<products::Model, String> {
        let db = &self.state.db;

        let category = string_to_product_category(&request.category)
            .ok_or("Invalid product category")?;

        let product = products::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set(request.name),
            image_url: Set(request.image_url),
            game: Set(request.game),
            expansion: Set(request.expansion),
            set_number: Set(request.set_number),
            category: Set(category),
            subcategory: Set(request.subcategory),
            metadata: Set(request.metadata),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
        };

        let product: products::Model = product.insert(db).await
            .map_err(|e| format!("Failed to create product: {}", e))
            .map(|model| model.into())?;
        /*
        let search_service = MeilisearchService::new(self.state.clone());
        
        match search_service.index_product(&product).await {
            Ok(_) => Ok(product),
            Err(e) => Err(format!("Failed to index product: {}", e)),
        }

         */

        Ok(product)
    }

    pub async fn update_product(
        &self,
        request: UpdateProductRequest,
    ) -> Result<products::Model, String> {
        let db = &self.state.db;

        let mut product: products::ActiveModel = products::Entity::find_by_id(request.id)
            .one(db)
            .await
            .map_err(|e| format!("Failed to fetch product: {}", e))?
            .ok_or("Product not found")?
            .into();
        
        if let Some(name) = request.name {
            product.name = Set(name);
        }

        if let Some(game) = request.game {
            product.game = Set(game);
        }

        if let Some(metadata) = request.metadata {
            product.metadata = Set(metadata);
        }
        
        product.image_url = Set(request.image_url);
        product.expansion = Set(request.expansion);
        product.updated_at = Set(chrono::Utc::now());

        product.update(db).await
            .map_err(|e| format!("Failed to update product: {}", e))
    }

    pub async fn delete_product(
        &self,
        id: Uuid
    ) -> Result<bool, String> {
        let db = &self.state.db;

        products::Entity::delete_by_id(id)
            .exec(db)
            .await
            .map_err(|e| format!("Failed to delete product: {}", e))
            .map(|_| true)
    }

    pub async fn get_product_by_id(
        &self,
        id: &Uuid,
    ) -> Result<Option<products::Model>, String> {
        let db = &self.state.db;

        products::Entity::find_by_id(*id)
            .one(db)
            .await
            .map_err(|e| format!("Failed to fetch product: {}", e))
    }

    pub async fn get_products(
        &self,
        offset: u64,
        limit: u64,
    ) -> Result<Vec<products::Model>, String> {
        let db = &self.state.db;

        products::Entity::find()
            .offset(offset)
            .limit(limit)
            .all(db)
            .await
            .map_err(|e| format!("Failed to fetch products: {}", e))
    }

    pub async fn get_number_of_products(
        &self,
    ) -> Result<u64, String> {
        let db = &self.state.db;

        products::Entity::find()
            .count(db)
            .await
            .map_err(|e| format!("Failed to count products: {}", e))
    }

    pub async fn upload_product_images(
        &self,
        user_id: Uuid,
        product_id: Uuid,
        request: ProductImageUploadRequest,
    ) -> Result<products::Model, String> {
        let db = &self.state.db;

        let product = products::Entity::find_by_id(product_id)
            .one(db)
            .await
            .map_err(|e| format!("Database error: {}", e))?
            .ok_or("Product not found")?;

        let mut product_active: products::ActiveModel = product.clone().into();

        let mut metadata = if let Ok(meta) = serde_json::from_value::<ProductMetadata>(product.metadata.clone()) {
            meta
        } else {
            ProductMetadata {
                images: Some(HashMap::new()),
                variants: Some(Vec::new()),
                primary_variant: None,
                variant_names: Some(HashMap::new()),
                additional: HashMap::new(),
            }
        };

        let mut images = metadata.images.unwrap_or_default();
        let mut variants = metadata.variants.unwrap_or_default();
        let mut variant_names = metadata.variant_names.unwrap_or_default();
        let mut primary_variant = metadata.primary_variant;

        let r2_service = R2Service::new(self.state.clone());

        for upload in request.uploads {
            let front_data = upload.front_image.and_then(|img| img.file_data);
            let back_data = upload.back_image.and_then(|img| img.file_data);

            if front_data.is_some() || back_data.is_some() {
                let variant_urls = r2_service.upload_product_variant_images(
                    product_id,
                    &product.game,
                    &upload.variant_name,
                    front_data.as_deref(),
                    back_data.as_deref(),
                    &user_id.to_string(),
                ).await.map_err(|e| format!("Failed to upload images: {}", e))?;

                let mut variant_images = ImageVariantData {
                    front: None,
                    back: None,
                };

                if let Some(front_urls) = variant_urls.front {
                    variant_images.front = Some(front_urls.medium);
                }

                if let Some(back_urls) = variant_urls.back {
                    variant_images.back = Some(back_urls.medium);
                }

                images.insert(upload.variant_id.clone(), variant_images);

                if !variants.contains(&upload.variant_id) {
                    variants.push(upload.variant_id.clone());
                }

                variant_names.insert(upload.variant_id.clone(), upload.variant_name);

                if upload.is_primary || primary_variant.is_none() {
                    primary_variant = Some(upload.variant_id);
                }
            }
        }

        let mut final_metadata = serde_json::Map::new();
        final_metadata.insert("images".to_string(), serde_json::to_value(images).unwrap());
        final_metadata.insert("variants".to_string(), serde_json::to_value(variants).unwrap());
        final_metadata.insert("variant_names".to_string(), serde_json::to_value(variant_names).unwrap());

        if let Some(primary) = &primary_variant {
            final_metadata.insert("primary_variant".to_string(), serde_json::to_value(primary).unwrap());

            if let Some(primary_front) = final_metadata.get("images")
                .and_then(|imgs| imgs.get(primary))
                .and_then(|variant| variant.get("front"))
                .and_then(|url| url.as_str()) {
                product_active.image_url = Set(Some(primary_front.to_string()));
            }
        }

        for (key, value) in metadata.additional {
            final_metadata.insert(key, value);
        }

        product_active.metadata = Set(serde_json::Value::Object(final_metadata));
        product_active.updated_at = Set(chrono::Utc::now());

        let updated_product = product_active.update(db).await
            .map_err(|e| format!("Failed to update product: {}", e))?;

        Ok(updated_product)
    }

    pub async fn delete_product_variant(
        &self,
        product_id: Uuid,
        variant_id: String,
    ) -> Result<products::Model, String> {
        let db = &self.state.db;

        let product = products::Entity::find_by_id(product_id)
            .one(db)
            .await
            .map_err(|e| format!("Database error: {}", e))?
            .ok_or("Product not found")?;

        let mut product_active: products::ActiveModel = product.clone().into();

        let mut metadata = serde_json::from_value::<ProductMetadata>(product.metadata.clone())
            .map_err(|e| format!("Invalid metadata: {}", e))?;

        let variant_exists = metadata.images
            .as_ref()
            .map(|images| images.contains_key(&variant_id))
            .unwrap_or(false);

        if !variant_exists {
            return Err(format!("Variant '{}' not found", variant_id));
        }

        let variant_count = metadata.variants.as_ref().map(|v| v.len()).unwrap_or(0);
        if variant_count <= 1 {
            return Err("Cannot delete the last remaining variant".to_string());
        }

        let r2_service = R2Service::new(self.state.clone());
        r2_service.delete_product_variant_images(
            product_id,
            &product.game,
            Some(&variant_id),
        ).await.map_err(|e| format!("Failed to delete images from storage: {}", e))?;

        if let Some(ref mut images) = metadata.images {
            images.remove(&variant_id);
        }

        if let Some(ref mut variants) = metadata.variants {
            variants.retain(|v| v != &variant_id);
        }

        if let Some(ref mut variant_names) = metadata.variant_names {
            variant_names.remove(&variant_id);
        }

        let mut needs_new_primary = false;
        if let Some(ref primary) = metadata.primary_variant {
            if primary == &variant_id {
                needs_new_primary = true;
            }
        }

        if needs_new_primary {
            metadata.primary_variant = metadata.variants
                .as_ref()
                .and_then(|v| v.first())
                .cloned();

            if let Some(new_primary) = &metadata.primary_variant {
                if let Some(new_primary_front) = metadata.images
                    .as_ref()
                    .and_then(|imgs| imgs.get(new_primary))
                    .and_then(|variant| variant.front.as_ref()) {
                    product_active.image_url = Set(Some(new_primary_front.clone()));
                }
            }
        }

        let mut final_metadata = serde_json::Map::new();

        if let Some(images) = metadata.images {
            final_metadata.insert("images".to_string(), serde_json::to_value(images).unwrap());
        }

        if let Some(variants) = metadata.variants {
            final_metadata.insert("variants".to_string(), serde_json::to_value(variants).unwrap());
        }

        if let Some(variant_names) = metadata.variant_names {
            final_metadata.insert("variant_names".to_string(), serde_json::to_value(variant_names).unwrap());
        }

        if let Some(primary_variant) = metadata.primary_variant {
            final_metadata.insert("primary_variant".to_string(), serde_json::to_value(primary_variant).unwrap());
        }

        for (key, value) in metadata.additional {
            final_metadata.insert(key, value);
        }

        product_active.metadata = Set(serde_json::Value::Object(final_metadata));
        product_active.updated_at = Set(chrono::Utc::now());

        let updated_product = product_active.update(db).await
            .map_err(|e| format!("Failed to update product: {}", e))?;

        Ok(updated_product)
    }
}