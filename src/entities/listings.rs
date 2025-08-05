use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use crate::entities::products::ProductCategory;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "listing_status")]
pub enum ListingStatus {
    #[sea_orm(string_value = "active")]
    Active,
    #[sea_orm(string_value = "sold")]
    Sold,
    #[sea_orm(string_value = "cancelled")]
    Cancelled,
    #[sea_orm(string_value = "expired")]
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "card_condition")]
pub enum Condition {
    #[sea_orm(string_value = "mint")]
    Mint,
    #[sea_orm(string_value = "near_mint")]
    NearMint,
    #[sea_orm(string_value = "lightly_played")]
    LightlyPlayed,
    #[sea_orm(string_value = "moderately_played")]
    ModeratelyPlayed,
    #[sea_orm(string_value = "heavily_played")]
    HeavilyPlayed,
    #[sea_orm(string_value = "damaged")]
    Damaged,
    #[sea_orm(string_value = "new")]
    New,
    #[sea_orm(string_value = "used")]
    Used,
    #[sea_orm(string_value = "sealed")]
    Sealed,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "listings")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub product_id: Uuid,
    pub seller_id: Uuid,
    pub price: i64,
    pub condition: Condition,
    pub quantity: i64,
    pub reserved_quantity: i64,
    pub status: ListingStatus,
    pub stripe_product_id: String,
    pub previous_stripe_product_id: Option<String>,
    pub image_url: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTimeUtc,
    pub updated_at: DateTimeUtc,
    pub deleted_at: Option<DateTimeUtc>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::products::Entity",
        from = "Column::ProductId",
        to = "super::products::Column::Id"
    )]
    Product,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::SellerId",
        to = "super::users::Column::Id"
    )]
    Seller,
}

impl Related<super::products::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Product.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Seller.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

impl Model {
    pub fn is_active(&self) -> bool {
        self.status == ListingStatus::Active
    }

    pub fn is_sold(&self) -> bool {
        self.status == ListingStatus::Sold
    }

    pub fn is_cancelled(&self) -> bool {
        self.status == ListingStatus::Cancelled
    }

    pub fn is_expired(&self) -> bool {
        self.status == ListingStatus::Expired
    }
}

impl Condition {
    pub fn valid_for_category(&self, category: &ProductCategory) -> bool {
        match (self, category) {
            (Condition::Mint | Condition::NearMint | Condition::LightlyPlayed |
            Condition::ModeratelyPlayed | Condition::HeavilyPlayed | Condition::Damaged,
                ProductCategory::Card) => true,
            (Condition::New | Condition::Used, ProductCategory::Accessory) => true,
            (Condition::Sealed, ProductCategory::Sealed) => true,
            _ => false,
        }
    }

    pub fn card_conditions() -> Vec<Condition> {
        vec![
            Condition::Mint,
            Condition::NearMint,
            Condition::LightlyPlayed,
            Condition::ModeratelyPlayed,
            Condition::HeavilyPlayed,
            Condition::Damaged,
        ]
    }

    pub fn accessory_conditions() -> Vec<Condition> {
        vec![Condition::New, Condition::Used]
    }
    
    pub fn sealed_conditions() -> Vec<Condition> {
        vec![Condition::Sealed]
    }
}

pub fn string_to_condition(condition: &str) -> Option<Condition> {
    match condition.to_lowercase().as_str() {
        "mint" => Some(Condition::Mint),
        "near_mint" => Some(Condition::NearMint),
        "lightly_played" => Some(Condition::LightlyPlayed),
        "moderately_played" => Some(Condition::ModeratelyPlayed),
        "heavily_played" => Some(Condition::HeavilyPlayed),
        "damaged" => Some(Condition::Damaged),
        "new" => Some(Condition::New),
        "used" => Some(Condition::Used),
        "sealed" => Some(Condition::Sealed),
        _ => None,
    }
}

pub fn get_valid_conditions_for_category(category: &ProductCategory) -> Vec<Condition> {
    match category {
        ProductCategory::Card => Condition::card_conditions(),
        ProductCategory::Accessory => Condition::accessory_conditions(),
        ProductCategory::Sealed => Condition::sealed_conditions(),
        ProductCategory::Other => Condition::accessory_conditions()
    }
}