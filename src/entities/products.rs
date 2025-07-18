use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "products")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub name: String,
    pub image_url: Option<String>,
    pub game: String,
    pub expansion: Option<String>,
    #[sea_orm(column_type = "Json")]
    pub metadata: JsonValue,
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

impl Model {
    pub fn get_metadata_string(&self, key: &str) -> Option<String> {
        self.metadata.get(key)?.as_str().map(|s| s.to_string())
    }
    
    pub fn get_metadata_number(&self, key: &str) -> Option<f64> {
        self.metadata.get(key)?.as_f64()
    }
    
    pub fn get_metadata_bool(&self, key: &str) -> Option<bool> {
        self.metadata.get(key)?.as_bool()
    }
    
    pub fn get_metadata_array(&self, key: &str) -> Option<Vec<String>> {
        self.metadata.get(key)?
            .as_array()?
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect::<Vec<_>>()
            .into()
    }
    
    pub fn get_metadata_keys(&self) -> Vec<String> {
        if let Some(obj) = self.metadata.as_object() {
            obj.keys().cloned().collect()
        } else {
            vec![]
        }
    }
    
    pub fn has_metadata_key(&self, key: &str) -> bool {
        self.metadata.get(key).is_some()
    }
}