use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use chrono::{DateTime, Utc};
use reqwest::{Client, Response};
use sea_orm::prelude::Decimal;
use serde::{Deserialize, Serialize};
use crate::app_state::AppState;
use crate::config::config::Config;

pub struct StripeClient {
    client: Arc<Client>,
    api_key: String,
    base_url: String,
}

impl StripeClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .pool_max_idle_per_host(3) // Conservative for Stripe
            .build()
            .expect("Failed to create Stripe client");

        Self {
            client: Arc::new(client),
            api_key: Config::get().stripe_key.clone(),
            base_url: "https://api.stripe.com/v1".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripeProduct {
    pub id: String,
    pub object: String,
    pub active: bool,
    pub created: u64,
    pub default_price: String,
    pub description: Option<String>,
    pub images: Vec<String>,
    pub marketing_features: Vec<MarketingFeature>,
    pub livemode: bool,
    pub metadata: HashMap<String, String>,
    pub name: String,
    pub package_dimensions: Option<PackageDimensions>,
    pub shippable: Option<bool>,
    pub statement_descriptor: Option<String>,
    pub tax_code: Option<String>,
    pub unit_label: Option<String>,
    pub updated: u64,
    pub url: Option<String>,
}

impl StripeProduct {
    pub fn created_at(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.created as i64, 0)
            .unwrap_or_else(|| Utc::now())
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.updated as i64, 0)
            .unwrap_or_else(|| Utc::now())
    }

    pub fn has_images(&self) -> bool {
        !self.images.is_empty()
    }

    pub fn is_shippable(&self) -> bool {
        self.shippable.unwrap_or(false)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketingFeature {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageDimensions {
    pub height: f64,
    pub length: f64,
    pub weight: f64,
    pub width: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripePrice {
    pub id: String,
    pub object: String,
    pub active: bool,
    pub billing_scheme: String,
    pub created: u64,
    pub currency: String,
    pub custom_unit_amount: Option<serde_json::Value>,
    pub livemode: bool,
    pub lookup_key: Option<String>,
    pub metadata: HashMap<String, String>,
    pub nickname: Option<String>,
    pub product: String,
    pub recurring: Option<serde_json::Value>,
    pub tax_behavior: Option<String>,
    pub tiers_mode: Option<String>,
    pub transform_quantity: Option<serde_json::Value>,
    pub r#type: String,
    pub unit_amount: Option<i64>,
    pub unit_amount_decimal: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductWithPrice {
    pub product: StripeProduct,
    pub price: StripePrice,
}

pub struct StripeService {
    state: AppState,
}

impl StripeService {
    pub fn new(state: AppState) -> Self {
        Self { state }
    }
    
    pub async fn create_stripe_product(
        product_name: &str,
        product_description: Option<&str>,
        price: i64,
    ) -> Result<StripeProduct, String> {
        let key: &str = Config::get().stripe_key.as_ref();
        let client = Client::new();

        let product_url = "https://api.stripe.com/v1/products";
        let mut product_params = vec![
            ("name", product_name.to_string()),
            ("default_price_data[unit_amount]", price.to_string()),
            ("default_price_data[currency]", "eur".to_string()),
        ];

        if let Some(desc) = product_description {
            product_params.push(("description", desc.to_string()));
        }

        let product_response = client
            .post(product_url)
            .header("Authorization", format!("Bearer {}", key))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .form(&product_params)
            .send()
            .await
            .map_err(|e| format!("Failed to send product creation request: {}", e))?;

        if !product_response.status().is_success() {
            let error_text = product_response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Stripe product creation error: {}", error_text));
        }

        let product: StripeProduct = product_response.json().await
            .map_err(|e| format!("Failed to parse product response: {}", e))?;

        Ok(product)
    }
    
    pub async fn get_stripe_product(
        product_id: &str,
    ) -> Result<StripeProduct, String> {
        let key: &str = Config::get().stripe_key.as_ref();
        let client = reqwest::Client::new();
        
        let url = format!("https://api.stripe.com/v1/products/{}", product_id);
        
        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", key))
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        if !response.status().is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Stripe product retrieval error: {}", error_text));
        }

        response.json::<StripeProduct>().await
            .map_err(|e| format!("Failed to parse product response: {}", e))
    }
    
    pub async fn delete_stripe_product(
        &self,
        product_id: &str,
    ) -> Result<(), String> {
        let key: &str = Config::get().stripe_key.as_ref();
        let client = reqwest::Client::new();

        let product_url = format!("https://api.stripe.com/v1/products/{}", product_id);
        
        let response = client
            .get(&product_url)
            .header("Authorization", format!("Bearer {}", key))
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await
            .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Stripe product retrieval error: {}", error_text));
        }
        
        let product = response.json::<StripeProduct>().await
            .map_err(|e| format!("Failed to parse product response: {}", e));
        
        let price_url = format!("https://api.stripe.com/v1/prices/{}", product?.default_price);

        let delete_price_response = client
            .delete(&price_url)
            .header("Authorization", format!("Bearer {}", key))
            .send()
            .await
            .map_err(|e| format!("Failed to send request: {}", e))?;

        if !delete_price_response.status().is_success() {
            let error_text = delete_price_response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Stripe product retrieval error: {}", error_text));
        }
        
        let delete_response = client
            .delete(&product_url)
            .header("Authorization", format!("Bearer {}", key))
            .send()
            .await
            .map_err(|e| format!("Failed to send delete request: {}", e))?;

        if !delete_response.status().is_success() {
            let error_text = delete_response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(format!("Stripe product deletion error: {}", error_text));
        }

        Ok(())
    }
}