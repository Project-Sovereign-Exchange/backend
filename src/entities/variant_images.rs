use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "variant_images")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub variant_id: Uuid,
    pub image_type: String,
    pub image_url: String,
    pub size: Option<String>,
    pub created_at: DateTimeWithTimeZone,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::product_variants::Entity",
        from = "Column::VariantId",
        to = "super::product_variants::Column::Id",
        on_delete = "Cascade"
    )]
    ProductVariant,
}

impl Related<super::product_variants::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProductVariant.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}