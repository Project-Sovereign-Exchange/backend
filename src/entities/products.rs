use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use serde_json::error::Category;
use crate::entities::listings::Condition;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "product_category")]
pub enum ProductCategory {
    #[sea_orm(string_value = "card")]
    Card,
    #[sea_orm(string_value = "sealed")]
    Sealed,
    #[sea_orm(string_value = "accessory")]
    Accessory,
    #[sea_orm(string_value = "other")]
    Other,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "products")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub game: String,
    pub set: Option<String>,
    pub category: ProductCategory,
    pub subcategory: Option<String>,
    #[sea_orm(column_type = "Json")]
    pub metadata: Option<JsonValue>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::listings::Entity")]
    Listings,
}

impl Related<super::listings::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Listings.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub fn string_to_product_category(category: &str) -> Option<ProductCategory> {
    match category.to_lowercase().as_str() {
        "card" => Some(ProductCategory::Card),
        "sealed" => Some(ProductCategory::Sealed),
        "accessory" => Some(ProductCategory::Accessory),
        _ => None,
    }
}