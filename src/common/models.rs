#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize, FromRow)]
pub struct Content {
    pub cid: u32,
    pub title: Option<String>,
    pub slug: Option<String>,
    pub created: u32,
    pub modified: u32,
    pub text: String,
    pub order: u32,
    pub authorId: u32,
    pub template: Option<String>,
    pub r#type: String,
    pub status: String,
    pub password: Option<String>,
    pub commentsNum: u32,
    pub allowComment: String,
    pub allowPing: String,
    pub allowFeed: String,
    pub parent: u32,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct Meta {
    pub mid: u32,
    pub name: Option<String>,
    pub slug: Option<String>,
    pub r#type: String,
    pub description: Option<String>,
    pub count: u32,
    pub order: u32,
    pub parent: u32,
}

#[derive(Serialize, Deserialize, FromRow)]
pub struct Field {
    pub cid: u32,
    pub name: String,
    pub r#type: String,
    pub str_value: Option<String>,
    pub int_value: u32,
    pub float_value: f64,
}

#[derive(Serialize, Deserialize)]
pub struct ContentWithFields {
    pub cid: u32,
    pub title: Option<String>,
    pub slug: Option<String>,
    pub created: u32,
    pub modified: u32,
    pub text: String,
    pub order: u32,
    pub authorId: u32,
    pub template: Option<String>,
    pub r#type: String,
    pub status: String,
    pub password: Option<String>,
    pub commentsNum: u32,
    pub allowComment: String,
    pub allowPing: String,
    pub allowFeed: String,
    pub parent: u32,
    pub fields: Vec<Field>,
}

impl From<Content> for ContentWithFields {
    fn from(content: Content) -> Self {
        Self {
            cid: content.cid,
            title: content.title,
            slug: content.slug,
            created: content.created,
            modified: content.modified,
            text: content.text,
            order: content.order,
            authorId: content.authorId,
            template: content.template,
            r#type: content.r#type,
            status: content.status,
            password: content.password,
            commentsNum: content.commentsNum,
            allowComment: content.allowComment,
            allowPing: content.allowPing,
            allowFeed: content.allowFeed,
            parent: content.parent,
            fields: vec![],
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct ContentWithMetasUsersFields {
    pub cid: u32,
    pub title: Option<String>,
    pub slug: Option<String>,
    pub created: u32,
    pub modified: u32,
    pub text: String,
    pub order: u32,
    pub authorId: u32,
    pub template: Option<String>,
    pub r#type: String,
    pub status: String,
    pub password: Option<String>,
    pub commentsNum: u32,
    pub allowComment: String,
    pub allowPing: String,
    pub allowFeed: String,
    pub parent: u32,
    pub screenName: Option<String>,
    pub group: String,
    pub categories: Vec<Meta>,
    pub tags: Vec<Meta>,
    pub fields: Vec<Field>,
}

impl From<Content> for ContentWithMetasUsersFields {
    fn from(content: Content) -> Self {
        Self {
            cid: content.cid,
            title: content.title,
            slug: content.slug,
            created: content.created,
            modified: content.modified,
            text: content.text,
            order: content.order,
            authorId: content.authorId,
            template: content.template,
            r#type: content.r#type,
            status: content.status,
            password: content.password,
            commentsNum: content.commentsNum,
            allowComment: content.allowComment,
            allowPing: content.allowPing,
            allowFeed: content.allowFeed,
            parent: content.parent,
            screenName: None,
            group: String::from(""),
            categories: vec![],
            tags: vec![],
            fields: vec![],
        }
    }
}
