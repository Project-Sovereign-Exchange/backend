use std::collections::HashMap;
use sea_orm::{ActiveModelTrait, EntityTrait, QuerySelect, Set, PaginatorTrait, JsonValue, QueryFilter, ColumnTrait};
use serde::{Deserialize, Serialize};
use crate::entities::products;
use crate::entities::product_variants;
use crate::handlers::marketplace::product_handler::{CreateProductRequest, CreateVariantRequest, ProductImageUploadRequest, ProductResponse};
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
    ) -> Result<ProductResponse, String> {
        let db = &self.state.db;

        let category = string_to_product_category(&request.category)
            .ok_or("Invalid product category")?;

        let product = products::ActiveModel {
            id: Set(Uuid::new_v4()),
            name: Set(request.name),
            description: Set(request.description),
            image_url: Set(request.base_image_url),
            game: Set(request.game),
            set: Set(request.set),
            category: Set(category),
            subcategory: Set(request.subcategory),
            metadata: Set(request.metadata),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
        };

        let product: products::Model = product.insert(db).await
            .map_err(|e| format!("Failed to create product: {}", e))
            .map(|model| model.into())?;

        let variants = self.create_product_variants(
            product.id,
            request.variants
        ).await.map_err(|e| format!("Failed to create product variants: {}", e))?;


        let search_service = MeilisearchService::new(self.state.clone());
        
        if let Err(e) = search_service.index_product(&product).await {
            return Err(format!("Failed to index product: {}", e));
        }

        Ok(ProductResponse {
            product,
            variants
        })
    }

    pub async fn create_product_variants(
        &self,
        product_id: Uuid,
        requests: Vec<CreateVariantRequest>,
    ) -> Result<Vec<product_variants::Model>, String> {
        let db = &self.state.db;

        let product = products::Entity::find_by_id(product_id)
            .one(db)
            .await
            .map_err(|e| format!("Failed to fetch product: {}", e))?
            .ok_or("Product not found")?;

        let mut variants = Vec::new();

        for request in requests {
            let variant = self.create_product_variant(
                product_id,
                request.name,
                request.set_number,
                request.is_primary,
            ).await?;

            variants.push(variant);
        }

        Ok(variants)
    }

    async fn create_product_variant(
        &self,
        product_id: Uuid,
        name: String,
        set_number: Option<String>,
        is_primary: bool,
    ) -> Result<product_variants::Model, String> {
        let db = &self.state.db;

        let variant = product_variants::ActiveModel {
            product_id: Set(product_id),
            name: Set(name),
            set_number: Set(set_number),
            is_primary: Set(is_primary),
            created_at: Set(chrono::Utc::now()),
            ..Default::default()
        };

        let variant = variant.insert(db).await
            .map_err(|e| format!("Failed to create product variant: {}", e))?;

        Ok(variant)
    }

    pub async fn get_product_variants(
        &self,
        product_id: &Uuid,
    ) -> Result<Vec<product_variants::Model>, String> {
        let db = &self.state.db;

        product_variants::Entity::find()
            .filter(product_variants::Column::ProductId.eq(*product_id))
            .all(db)
            .await
            .map_err(|e| format!("Failed to fetch product variants: {}", e))
    }

    /*
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
        product.set = Set(request.set);
        product.updated_at = Set(chrono::Utc::now());

        product.update(db).await
            .map_err(|e| format!("Failed to update product: {}", e))
    }
     */

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

    /*
    pub async fn delete_product_variant(
        &self,
        product_id: Uuid,
        variant_id: String,
    ) -> Result<products::Model, String> {

    }

     */

    pub async fn upload_product_images(
        &self,
        user_id: &Uuid,
        product_id: &Uuid,
        request: ProductImageUploadRequest,
    ) -> Result<(), String> {
        let db = &self.state.db;

        let product = products::Entity::find_by_id(*product_id)
            .one(db)
            .await
            .map_err(|e| format!("Failed to fetch product: {}", e))?
            .ok_or("Product not found")?;


        let r2_service = R2Service::new(self.state.clone());

        for upload in request.uploads {
            let front_data = upload.front_image.and_then(|img| img.file_data);
            let back_data = upload.back_image.and_then(|img| img.file_data);

            let variant_urls = r2_service.upload_product_variant_images(
                product_id,
                user_id,
                &product.game,
                &upload.variant_name,
                front_data.as_deref(),
                back_data.as_deref(),
            ).await.map_err(|e| format!("Failed to upload front image: {}", e))?;
        }

        Ok(())
    }
}