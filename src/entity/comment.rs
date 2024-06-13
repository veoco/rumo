use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "typecho_comments")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub coid: u32,
    pub cid: u32,
    pub created: u32,
    pub author: Option<String>,
    #[sea_orm(column_name = "authorId")]
    pub author_id: u32,
    #[sea_orm(column_name = "ownerId")]
    pub owner_id: u32,
    pub mail: Option<String>,
    pub url: Option<String>,
    pub ip: Option<String>,
    pub agent: Option<String>,
    #[sea_orm(column_type = "Text", nullable)]
    pub text: Option<String>,
    pub r#type: String,
    pub status: String,
    pub parent: u32,
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
        belongs_to = "super::user::Entity",
        from = "Column::AuthorId",
        to = "super::user::Column::Uid"
    )]
    Author,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::OwnerId",
        to = "super::user::Column::Uid"
    )]
    Owner,
    #[sea_orm(belongs_to = "Entity", from = "Column::Parent", to = "Column::Coid")]
    Parent,
    #[sea_orm(has_many = "Entity")]
    Children,
}

impl Related<super::content::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Content.def()
    }
}

impl Related<super::user::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Author.def()
    }
}

impl Related<Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Parent.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}