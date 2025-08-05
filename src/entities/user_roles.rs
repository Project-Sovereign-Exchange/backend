use sea_orm::entity::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum)]
#[sea_orm(rs_type = "String", db_type = "Enum", enum_name = "user_role_type")]
pub enum UserRoleType {
    #[sea_orm(string_value = "buyer")]
    Buyer,
    #[sea_orm(string_value = "seller")]
    Seller,
    #[sea_orm(string_value = "moderator")]
    Moderator,
    #[sea_orm(string_value = "admin")]
    Admin,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user_roles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: Uuid,
    pub user_id: Uuid,
    pub role: UserRoleType,
    pub granted_at: DateTimeWithTimeZone,
    pub granted_by: Option<Uuid>,
    pub is_active: bool,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::UserId",
        to = "super::users::Column::Id"
    )]
    Users,
    #[sea_orm(
        belongs_to = "super::users::Entity",
        from = "Column::GrantedBy",
        to = "super::users::Column::Id"
    )]
    GrantedBy,
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Users.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}