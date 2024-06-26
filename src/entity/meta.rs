use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "typecho_metas")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub mid: u32,
    pub name: Option<String>,
    pub slug: Option<String>,
    pub r#type: String,
    pub description: Option<String>,
    pub count: u32,
    pub order: u32,
    pub parent: u32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl Related<super::meta::Entity> for Entity {
    fn to() -> RelationDef {
        super::relationship::Relation::Content.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::relationship::Relation::Meta.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
