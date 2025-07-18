use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use crate::entities::products;
use crate::handlers::marketplace::product_handler::{CreateProductRequest, UpdateProductRequest};
use crate::services::integrations::db_service::DbService;
use uuid::Uuid;

pub struct ProductService;

impl ProductService {
    pub async fn create_product(
        request: CreateProductRequest,
    ) -> Result<products::Model, String> {
        let db = DbService::get().await
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        let product = products::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            name: Set(request.name),
            image_url: Set(request.image_url),
            game: Set(request.game),
            expansion: Set(request.expansion),
            metadata: Set(request.metadata),
            created_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
        };

        product.insert(db).await
            .map_err(|e| format!("Failed to create product: {}", e))
    }

    pub async fn update_product(
        request: UpdateProductRequest,
    ) -> Result<products::Model, String> {
        let db = DbService::get().await
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

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

    pub async fn delete_product(id: uuid::Uuid) -> Result<bool, String> {
        let db = DbService::get().await
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        products::Entity::delete_by_id(id)
            .exec(db)
            .await
            .map_err(|e| format!("Failed to delete product: {}", e))
            .map(|_| true)
    }

    pub async fn get_product_by_id(
        id: &Uuid,
    ) -> Result<Option<products::Model>, String> {
        let db = DbService::get().await
            .map_err(|e| format!("Failed to get database connection: {}", e))?;

        products::Entity::find_by_id(*id)
            .one(db)
            .await
            .map_err(|e| format!("Failed to fetch product: {}", e))
    }
}