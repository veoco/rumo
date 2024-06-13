use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "typecho_contents")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub cid: u32,
    pub title: Option<String>,
    #[sea_orm(unique)]
    pub slug: Option<String>,
    pub created: u32,
    pub modified: u32,
    #[sea_orm(column_type = "Text", nullable)]
    pub text: Option<String>,
    pub order: u32,
    #[sea_orm(column_name = "authorId")]
    pub author_id: u32,
    pub template: Option<String>,
    pub r#type: String,
    pub status: String,
    pub password: Option<String>,
    #[sea_orm(column_name = "commentsNum")]
    pub comments_num: u32,
    #[sea_orm(column_name = "allowComment")]
    pub allow_comment: String,
    #[sea_orm(column_name = "allowPing")]
    pub allow_ping: String,
    #[sea_orm(column_name = "allowFeed")]
    pub allow_feed: String,
    pub parent: u32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::comment::Entity")]
    Comment,
    #[sea_orm(
        belongs_to = "super::user::Entity",
        from = "Column::AuthorId",
        to = "super::user::Column::Uid"
    )]
    Author,
    #[sea_orm(belongs_to = "Entity", from = "Column::Parent", to = "Column::Cid")]
    Parent,
    #[sea_orm(has_many = "Entity")]
    Children,
    #[sea_orm(has_many = "super::field::Entity")]
    Field,
}

impl Related<super::comment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Comment.def()
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

impl Related<super::field::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Field.def()
    }
}

impl Related<super::meta::Entity> for Entity {
    fn to() -> RelationDef {
        super::relationship::Relation::Meta.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::relationship::Relation::Content.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
