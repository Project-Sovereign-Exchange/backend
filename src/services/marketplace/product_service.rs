use std::sync::Arc;
use sea_orm::{ActiveModelTrait, EntityTrait, QuerySelect, Set, PaginatorTrait};
use crate::entities::products;
use crate::handlers::marketplace::product_handler::{CreateProductRequest, UpdateProductRequest};
use uuid::Uuid;
use crate::app_state::AppState;
use crate::entities::products::string_to_product_category;
use crate::services::integrations::meilisearch_service::MeilisearchService;

pub struct ProductService {
    pub state: AppState,
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
        
        let search_service = MeilisearchService::new(self.state.clone());
        
        match search_service.index_product(&product).await {
            Ok(_) => Ok(product),
            Err(e) => Err(format!("Failed to index product: {}", e)),
        }
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
}