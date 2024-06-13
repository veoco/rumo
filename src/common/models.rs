use serde::{Deserialize, Serialize};

use crate::entity::{content, field::Model as Field, meta::Model as Meta};

#[derive(Serialize, Deserialize)]
pub struct ContentWithFields {
    pub cid: u32,
    pub title: Option<String>,
    pub slug: Option<String>,
    pub created: u32,
    pub modified: u32,
    pub text: Option<String>,
    pub order: u32,
    pub author_id: u32,
    pub template: Option<String>,
    pub r#type: String,
    pub status: String,
    pub password: Option<String>,
    pub comments_num: u32,
    pub allow_comment: String,
    pub allow_ping: String,
    pub allow_feed: String,
    pub parent: u32,
    pub fields: Vec<Field>,
}

impl From<content::Model> for ContentWithFields {
    fn from(content: content::Model) -> Self {
        Self {
            cid: content.cid,
            title: content.title,
            slug: content.slug,
            created: content.created,
            modified: content.modified,
            text: content.text,
            order: content.order,
            author_id: content.author_id,
            template: content.template,
            r#type: content.r#type,
            status: content.status,
            password: content.password,
            comments_num: content.comments_num,
            allow_comment: content.allow_comment,
            allow_ping: content.allow_ping,
            allow_feed: content.allow_feed,
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
    pub text: Option<String>,
    pub order: u32,
    pub author_id: u32,
    pub template: Option<String>,
    pub r#type: String,
    pub status: String,
    pub password: Option<String>,
    pub comments_num: u32,
    pub allow_comment: String,
    pub allow_ping: String,
    pub allow_feed: String,
    pub parent: u32,

    pub screen_name: Option<String>,
    pub group: String,
    pub categories: Vec<Meta>,
    pub tags: Vec<Meta>,
    pub fields: Vec<Field>,
}

impl From<content::Model> for ContentWithMetasUsersFields {
    fn from(content: content::Model) -> Self {
        Self {
            cid: content.cid,
            title: content.title,
            slug: content.slug,
            created: content.created,
            modified: content.modified,
            text: content.text,
            order: content.order,
            author_id: content.author_id,
            template: content.template,
            r#type: content.r#type,
            status: content.status,
            password: content.password,
            comments_num: content.comments_num,
            allow_comment: content.allow_comment,
            allow_ping: content.allow_ping,
            allow_feed: content.allow_feed,
            parent: content.parent,
            screen_name: None,
            group: "visitor".to_string(),
            categories: vec![],
            tags: vec![],
            fields: vec![],
        }
    }
}
