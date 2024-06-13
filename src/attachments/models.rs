use serde::{Deserialize, Serialize};

use super::de::from_str;
use crate::entity::content;

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
    pub cid: u32,
    pub created: u32,
    pub modified: u32,
    pub name: String,
    pub path: String,
    pub size: u64,
    pub r#type: String,
    pub mime: String,
}

impl From<content::Model> for AttachmentInfo {
    fn from(content: content::Model) -> Self {
        let text = content.text.unwrap_or("".to_string());
        if let Ok(at) = from_str::<AttachmentText>(&text) {
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
