use sea_orm::ActiveModelTrait;
use crate::entities::listings;
use crate::entities::listings::{string_to_card_condition, ListingStatus};
use crate::handlers::marketplace::listing_handler::CreateListingRequest;
use crate::services::account::user_service::UserService;
use crate::services::integrations::db_service::DbService;
use crate::services::integrations::stripe_service::StripeService;
use crate::services::marketplace::product_service::ProductService;

pub struct ListingService;

impl ListingService {
    pub async fn create_listing(
        user_id: uuid::Uuid,
        request: CreateListingRequest,
    ) -> Result<listings::Model, String> {
        UserService::get_user_by_id(&user_id)
            .await
            .map_err(|e| format!("Failed to fetch user: {}", e))?
            .ok_or_else(|| "User not found".to_string())?;
        
        let product = ProductService::get_product_by_id(&request.product_id)
            .await
            .map_err(|e| format!("Failed to fetch product: {}", e))?
            .ok_or_else(|| "Product not found".to_string())?;
        
        let stripe_product = StripeService::create_stripe_product(
            &product.name,
            request.description.as_deref(),
            request.price
        ).await
        .map_err(|e| format!("Failed to create Stripe product: {}", e))?;
        
        let condition = string_to_card_condition(
            &request.condition
        ).ok_or_else(|| "condition not defined".to_string())?;
        
        let listing = listings::Model {
            id: uuid::Uuid::new_v4(),
            product_id: request.product_id,
            seller_id: user_id,
            price: request.price,
            condition,
            quantity: request.quantity,
            reserved_quantity: 0,
            status: ListingStatus::Active,
            stripe_product_id: stripe_product.product.id,
            previous_stripe_product_id: None,
            image_url: request.image_url,
            description: request.description,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted_at: None,
        };

        let db = DbService::get().await
            .map_err(|e| format!("Failed to get database connection: {}", e))?;
        
        listings::ActiveModel::from(listing.clone())
            .insert(db)
            .await
            .map_err(|e| format!("Failed to create listing: {}", e))?;
        
        Ok(listing)
    }
}