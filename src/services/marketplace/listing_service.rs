use sea_orm::{ActiveModelTrait, EntityTrait, Set};
use uuid::Uuid;
use crate::app_state::AppState;
use crate::entities::listings;
use crate::entities::listings::{string_to_condition, ListingStatus};
use crate::handlers::marketplace::listing_handler::{CreateListingRequest, UpdateListingRequest};
use crate::services::account::user_service::UserService;
use crate::services::integrations::stripe_service::StripeService;
use crate::services::marketplace::product_service::ProductService;

pub struct ListingService {
    pub state: AppState,
}

impl ListingService {
    
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
    pub async fn create_listing(
        &self,
        user_id: Uuid,
        request: CreateListingRequest,
    ) -> Result<listings::Model, String> {
        let user_service = UserService::new(self.state.clone());

        user_service.get_user_by_id(&user_id)
            .await
            .map_err(|e| format!("Failed to fetch user: {}", e))?
            .ok_or_else(|| "User not found".to_string())?;

        let product_service = ProductService::new(self.state.clone());
        
        let product = product_service.get_product_by_id(&request.product_id)
            .await
            .map_err(|e| format!("Failed to fetch product: {}", e))?
            .ok_or_else(|| "Product not found".to_string())?;
        
        let stripe_product = StripeService::create_stripe_product(
            &product.name,
            request.description.as_deref(),
            request.price
        ).await
        .map_err(|e| format!("Failed to create Stripe product: {}", e))?;
        
        let condition = string_to_condition(
            &request.condition
        ).ok_or_else(|| "condition not defined".to_string())?;
        
        let listing = listings::Model {
            id: Uuid::new_v4(),
            product_id: request.product_id,
            seller_id: user_id,
            price: request.price,
            condition,
            quantity: request.quantity,
            reserved_quantity: 0,
            status: ListingStatus::Active,
            stripe_product_id: stripe_product.id,
            previous_stripe_product_id: None,
            image_url: request.image_url,
            description: request.description,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            deleted_at: None,
        };

        let db = &self.state.db;
        
        listings::ActiveModel::from(listing.clone())
            .insert(db)
            .await
            .map_err(|e| format!("Failed to create listing: {}", e))?;
        
        Ok(listing)
    }

    pub async fn update_listing(
        &self,
        user_id: Uuid,
        request: UpdateListingRequest,
    ) -> Result<listings::Model, String> {
        let db = &self.state.db;

        let listing = listings::Entity::find_by_id(request.id)
            .one(db)
            .await
            .map_err(|e| format!("Failed to fetch listing: {}", e))?
            .ok_or("Listing not found".to_string())?;

        if listing.seller_id != user_id {
            return Err("You are not the seller of this listing".to_string());
        }

        let mut listing: listings::ActiveModel = listing.into();

        if let Some(price) = request.price {
            listing.price = Set(price);
        }

        if let Some(condition) = request.condition {
            let card_condition = string_to_condition(&condition)
                .ok_or_else(|| "Invalid condition".to_string())?;
            listing.condition = Set(card_condition);
        }

        if let Some(quantity) = request.quantity {
            listing.quantity = Set(quantity);
        }

        if let Some(image_url) = request.image_url {
            listing.image_url = Set(Some(image_url));
        }

        if let Some(description) = request.description {
            listing.description = Set(Some(description));
        }

        listing.updated_at = Set(chrono::Utc::now());

        listing.update(db)
            .await
            .map_err(|e| format!("Failed to update listing: {}", e))
    }

    pub async fn get_listing(
        &self,
        id: Uuid
    ) -> Result<listings::Model, String> {
        println!("Fetching listing with ID: {}", id);
        
        let db = &self.state.db;

        listings::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| format!("Failed to fetch listing: {}", e))?
            .ok_or("Listing not found".to_string())
    }

    pub async fn delete_listing(
        &self,
        user_id: Uuid,
        id: Uuid
    ) -> Result<bool, String> {
        let db = &self.state.db;

        let listing = listings::Entity::find_by_id(id)
            .one(db)
            .await
            .map_err(|e| format!("Failed to fetch listing: {}", e))?
            .ok_or("Listing not found".to_string())?;
        
        if listing.seller_id != user_id {
            return Err("You are not the seller of this listing".to_string());
        }
        
        if listing.deleted_at.is_some() {
            return Ok(false);
        }

        let stripe_service = StripeService::new(self.state.clone());

        stripe_service.delete_stripe_product(&listing.stripe_product_id)
            .await
            .map_err(|e| format!("Failed to delete StripeProduct: {}", e))?;

        let mut listing: listings::ActiveModel = listing.into();
        listing.deleted_at = Set(Some(chrono::Utc::now()));

        let deleted = listing.update(db)
            .await
            .map_err(|e| format!("Failed to delete listing: {}", e))?;

        Ok(deleted.deleted_at.is_some())
    }
}