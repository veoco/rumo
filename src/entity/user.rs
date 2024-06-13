use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "typecho_users")]
pub struct Model {
    #[sea_orm(primary_key)]
    #[serde(skip_deserializing)]
    pub uid: u32,
    #[sea_orm(unique)]
    pub name: Option<String>,
    pub password: Option<String>,
    #[sea_orm(unique)]
    pub mail: Option<String>,
    pub url: Option<String>,
    #[sea_orm(column_name = "screenName")]
    pub screen_name: Option<String>,
    pub created: u32,
    pub activated: u32,
    pub logged: u32,
    pub group: String,
    #[sea_orm(column_name = "authCode")]
    pub auth_code: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::comment::Entity")]
    Comment,
    #[sea_orm(has_many = "super::comment::Entity")]
    PostComment,
    #[sea_orm(has_many = "super::content::Entity")]
    Content,
    #[sea_orm(has_many = "super::option::Entity")]
    Option,
}

impl Related<super::comment::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Comment.def()
    }
}

impl Related<super::content::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Content.def()
    }
}

impl Related<super::option::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Option.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
