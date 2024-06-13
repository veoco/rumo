use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Serialize, Deserialize, Validate)]
pub struct AttachmentsQuery {
    #[validate(range(min = 1, message = "page must greater than 1"))]
    pub page: Option<u64>,
    #[validate(range(min = 1, message = "page_size must greater than 1"))]
    pub page_size: Option<u64>,
    #[validate(length(min = 1, max = 13, message = "order_by length must greater than 1"))]
    pub order_by: Option<String>,
    pub private: Option<bool>,
}

#[derive(Serialize, Deserialize, Validate)]
pub struct AttachmentCreate {
    #[validate(range(min = 1, message = "cid must greater than 1"))]
    pub cid: u32,
}