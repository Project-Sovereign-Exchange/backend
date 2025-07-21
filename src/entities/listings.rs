use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

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
pub enum CardCondition {
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
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "listings")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: Uuid,
    pub product_id: Uuid,
    pub seller_id: Uuid,
    pub price: i64,
    pub condition: CardCondition,
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

pub fn string_to_card_condition(condition: &str) -> Option<CardCondition> {
    match condition.to_lowercase().as_str() {
        "mint" => Some(CardCondition::Mint),
        "near_mint" => Some(CardCondition::NearMint),
        "lightly_played" => Some(CardCondition::LightlyPlayed),
        "moderately_played" => Some(CardCondition::ModeratelyPlayed),
        "heavily_played" => Some(CardCondition::HeavilyPlayed),
        "damaged" => Some(CardCondition::Damaged),
        _ => None,
    }
}