use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "typecho_relationships")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    #[serde(skip_deserializing)]
    pub cid: u32,
    #[sea_orm(primary_key, auto_increment = false)]
    #[serde(skip_deserializing)]
    pub mid: u32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::content::Entity",
        from = "Column::Cid",
        to = "super::content::Column::Cid"
    )]
    Content,
    #[sea_orm(
        belongs_to = "super::meta::Entity",
        from = "Column::Mid",
        to = "super::meta::Column::Mid"
    )]
    Meta,
}

impl ActiveModelBehavior for ActiveModel {}
