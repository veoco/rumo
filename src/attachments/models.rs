#![allow(non_snake_case)]
use serde::{Deserialize, Serialize};
use validator::Validate;

use super::de::from_str;
use crate::common::models::Content;

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct AttachmentText {
    pub name: String,
    pub path: String,
    pub size: u64,
    pub r#type: String,
    pub mime: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct AttachmentInfo {
    pub cid: i32,
    pub created: i32,
    pub modified: i32,
    pub name: String,
    pub path: String,
    pub size: u64,
    pub r#type: String,
    pub mime: String,
}

impl From<Content> for AttachmentInfo {
    fn from(content: Content) -> Self {
        if let Ok(at) = from_str::<AttachmentText>(&content.text) {
            Self {
                cid: content.cid,
                created: content.created,
                modified: content.modified,
                name: at.name,
                path: at.path,
                size: at.size,
                r#type: at.r#type,
                mime: at.mime,
            }
        } else {
            Self {
                cid: 0,
                created: content.created,
                modified: content.modified,
                name: "".to_string(),
                path: "".to_string(),
                size: 0,
                r#type: "".to_string(),
                mime: "".to_string(),
            }
        }
    }
}

#[derive(Serialize, Deserialize, Validate)]
pub struct AttachmentsQuery {
    #[validate(range(min = 1, message = "page must greater than 1"))]
    pub page: Option<i32>,
    #[validate(range(min = 1, message = "page_size must greater than 1"))]
    pub page_size: Option<i32>,
    #[validate(length(min = 1, max = 13, message = "order_by length must greater than 1"))]
    pub order_by: Option<String>,
    pub private: Option<bool>,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct AttachmentCreate {
    #[validate(range(min = 1, message = "cid must greater than 1"))]
    pub cid: i32,
}
