use meilisearch_sdk::client::Client;
use meilisearch_sdk::search::SearchResults;
use sea_orm::sqlx::ColumnIndex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::app_state::AppState;
use crate::entities::{products, listings};

#[derive(Serialize, Deserialize)]
pub struct SearchableProduct {
    pub id: String,
    pub name: String,
    pub game: Option<String>,
    pub expansion: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct SearchableListing {
    pub id: String,
    pub product_name: String,
    pub price: f64,
    pub condition: String,
    pub game: Option<String>,
    pub expansion: Option<String>,
}

pub struct MeilisearchService {
    state: AppState,
}

impl MeilisearchService {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }

    pub async fn index_product(&self, product: &products::Model) -> Result<(), String> {
        let searchable_product = SearchableProduct {
            id: product.id.to_string(),
            name: product.name.clone(),
            game: Some(product.game.clone()),
            expansion: product.expansion.clone(),
            metadata: Some(product.metadata.clone()),
        };

        let products_index = self.state.meilisearch_client.as_ref().clone().index("products");
        products_index
            .add_documents(&[searchable_product], Some("id"))
            .await
            .map_err(|e| format!("Failed to index product: {}", e))?;

        Ok(())
    }

    pub async fn index_listing(&self, listing: &listings::Model, product: &products::Model) -> Result<(), String> {
        let searchable_listing = SearchableListing {
            id: listing.id.to_string(),
            product_name: product.name.clone(),
            price: listing.price as f64,
            condition: format!("{:?}", listing.condition),
            game: Some(product.game.clone()),
            expansion: product.expansion.clone(),
        };

        let listings_index = self.state.meilisearch_client.as_ref().clone().index("listings");
        listings_index
            .add_documents(&[searchable_listing], Some("id"))
            .await
            .map_err(|e| format!("Failed to index listing: {}", e))?;

        Ok(())
    }

    pub async fn search_products(&self, query: &str, filters: Option<&str>) -> Result<SearchResults<SearchableProduct>, String> {
        let products_index = self.state.meilisearch_client.as_ref().clone().index("products");
        let mut search = products_index.search();

        search.with_query(query);
        if let Some(filter) = filters {
            search.with_filter(filter);
        }

        search.execute::<SearchableProduct>()
            .await
            .map_err(|e| format!("Search failed: {}", e))
    }

    pub async fn search_listings(&self, query: &str, filters: Option<&str>) -> Result<SearchResults<SearchableListing>, String> {
        let listings_index = self.state.meilisearch_client.as_ref().clone().index("listings");
        let mut search = listings_index.search();

        search.with_query(query);
        if let Some(filter) = filters {
            search.with_filter(filter);
        }

        search.execute::<SearchableListing>()
            .await
            .map_err(|e| format!("Search failed: {}", e))
    }

    pub async fn setup_indexes(&self) -> Result<(), String> {
        let client = self.state.meilisearch_client.as_ref().clone();
        
        // Setup products index
        let products_index = client.index("products");
        products_index
            .set_filterable_attributes(["game", "expansion"])
            .await
            .map_err(|e| format!("Failed to set filterable attributes for products: {}", e))?;

        products_index
            .set_searchable_attributes(["name", "game", "expansion"])
            .await
            .map_err(|e| format!("Failed to set searchable attributes for products: {}", e))?;

        // Setup listings index
        let listings_index = client.index("listings");
        listings_index
            .set_filterable_attributes(["game", "expansion", "condition", "price"])
            .await
            .map_err(|e| format!("Failed to set filterable attributes for listings: {}", e))?;

        listings_index
            .set_searchable_attributes(["product_name", "game", "expansion"])
            .await
            .map_err(|e| format!("Failed to set searchable attributes for listings: {}", e))?;

        Ok(())
    }
}