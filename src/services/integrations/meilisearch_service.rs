use meilisearch_sdk::client::Client;
use meilisearch_sdk::search::SearchResults;
use sea_orm::EntityTrait;
use sea_orm::sqlx::ColumnIndex;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::app_state::AppState;
use crate::entities::{products, listings};
use crate::entities::products::ProductCategory;

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchableProduct {
    pub id: String,
    pub name: String,
    pub game: Option<String>,
    pub set: Option<String>,
    pub category: String,
    pub subcategory: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchableListing {
    pub id: String,
    pub product_name: String,
    pub price: f64,
    pub condition: String,
    pub game: Option<String>,
    pub set: Option<String>,
}

pub struct MeilisearchService {
    state: AppState,
}

impl MeilisearchService {
    pub fn new(state: AppState) -> Self { Self { state } }

    pub async fn validate_indexes(&self) -> Result<(), String> {
        let client = self.state.meilisearch_client.as_ref();

        if let Err(e) = client.get_index("products").await {
            self.setup_products_index().await?;
        }

        if let Err(e) = client.get_index("listings").await {
            self.setup_listings_index().await?;
        }

        Ok(())
    }

    pub async fn index_product(&self, product: &products::Model) -> Result<(), String> {
        let searchable_product = SearchableProduct {
            id: product.id.to_string(),
            name: product.name.clone(),
            game: Some(product.game.clone()),
            set: product.set.clone(),
            category: self.category_to_string(&product.category),
            subcategory: product.subcategory.clone(),
            metadata: product.metadata.clone(),
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
            set: product.set.clone(),
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

    pub async fn search_products_paginated(
        &self,
        query: &str,
        filters: Option<&str>,
        offset: usize,
        limit: usize,
        sort: Option<&[&str]>
    ) -> Result<SearchResults<SearchableProduct>, String> {
        let products_index = self.state.meilisearch_client.index("products");
        let mut search = products_index.search();

        search
            .with_query(query)
            .with_offset(offset)
            .with_limit(limit);

        if let Some(filter) = filters {
            search.with_filter(filter);
        }

        if let Some(sort_params) = sort {
            search.with_sort(sort_params);
        }

        search.execute::<SearchableProduct>()
            .await
            .map_err(|e| format!("Search failed: {}", e))
    }

    pub async fn search_products_by_category(
        &self,
        query: &str,
        category: Option<&ProductCategory>,
        game: Option<&str>,
        offset: usize,
        limit: usize,
    ) -> Result<SearchResults<SearchableProduct>, String> {
        let mut filters = Vec::new();

        if let Some(cat) = category {
            filters.push(format!("category = \"{}\"", self.category_to_string(cat)));
        }

        if let Some(g) = game {
            filters.push(format!("game = \"{}\"", g));
        }

        let filter_string = if filters.is_empty() {
            None
        } else {
            Some(filters.join(" AND "))
        };

        self.search_products_paginated(
            query,
            filter_string.as_deref(),
            offset,
            limit,
            None
        ).await
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

    pub async fn search_listings_paginated(
        &self,
        query: &str,
        filters: Option<&str>,
        offset: usize,
        limit: usize,
        sort: Option<&[&str]>
    ) -> Result<SearchResults<SearchableListing>, String> {
        let listings_index = self.state.meilisearch_client.index("listings");
        let mut search = listings_index.search();

        search
            .with_query(query)
            .with_offset(offset)
            .with_limit(limit);

        if let Some(filter) = filters {
            search.with_filter(filter);
        }

        if let Some(sort_params) = sort {
            search.with_sort(sort_params);
        }

        search.execute::<SearchableListing>()
            .await
            .map_err(|e| format!("Search failed: {}", e))
    }

    pub fn get_available_categories(&self) -> Vec<&'static str> {
        vec!["card", "sealed", "accessory", "other"]
    }

    fn category_to_string(&self, category: &ProductCategory) -> String {
        match category {
            ProductCategory::Card => "card".to_string(),
            ProductCategory::Sealed => "sealed".to_string(),
            ProductCategory::Accessory => "accessory".to_string(),
            ProductCategory::Other => "other".to_string(),
        }
    }

    pub fn string_to_category(&self, category_str: &str) -> Option<ProductCategory> {
        products::string_to_product_category(category_str)
    }

    async fn setup_products_index(&self) -> Result<(), String> {
        let client = self.state.meilisearch_client.as_ref().clone();
        let products_index = client.index("products");

        products_index
            .set_filterable_attributes(["game", "set", "category", "subcategory"])
            .await
            .map_err(|e| format!("Failed to set filterable attributes for products: {}", e))?;

        products_index
            .set_searchable_attributes(["name", "game", "set", "category", "subcategory"])
            .await
            .map_err(|e| format!("Failed to set searchable attributes for products: {}", e))?;

        products_index
            .set_sortable_attributes(["name", "game", "category"])
            .await
            .map_err(|e| format!("Failed to set sortable attributes for products: {}", e))?;

        Ok(())
    }

    async fn setup_listings_index(&self) -> Result<(), String> {
        let client = self.state.meilisearch_client.as_ref().clone();
        let listings_index = client.index("listings");

        listings_index
            .set_filterable_attributes(["game", "set", "condition", "price", "language"])
            .await
            .map_err(|e| format!("Failed to set filterable attributes for listings: {}", e))?;

        listings_index
            .set_searchable_attributes(["product_name", "game", "set"])
            .await
            .map_err(|e| format!("Failed to set searchable attributes for listings: {}", e))?;

        Ok(())
    }

    //used for debugging and initial indexing, will be moved to CLI later
    async fn index_all_products(&self) -> Result<(), String> {
        let products = products::Entity::find().all(&self.state.db)
            .await
            .map_err(|e| format!("Failed to fetch products: {}", e))?;

        for product in products {
            self.index_product(&product).await?;
        }

        Ok(())
    }
}