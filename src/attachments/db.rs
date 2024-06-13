use sea_orm::*;

use crate::common::db as common_db;
use crate::common::errors::FieldError;
use crate::entity::{content, content::Entity as Content};
use crate::AppState;

pub async fn create_attachment_with_params(
    state: &AppState,
    name: &str,
    now: u32,
    text: &str,
    uid: u32,
) -> Result<content::ActiveModel, FieldError> {
    content::ActiveModel {
        r#type: Set("attachment".to_string()),
        title: Set(Some(name.to_owned())),
        slug: Set(Some(name.to_owned())),
        created: Set(now),
        modified: Set(now),
        text: Set(Some(text.to_owned())),
        author_id: Set(uid),
        ..Default::default()
    }
    .save(&state.conn)
    .await
    .map_err(|_| FieldError::DatabaseFailed("create attachment failed".to_string()))
}

pub async fn modify_attachment_by_cid_with_params(
    state: &AppState,
    cid: u32,
    name: &str,
    now: u32,
    text: &str,
) -> Result<content::Model, FieldError> {
    let exist_attachment = common_db::get_content_by_cid(state, cid).await?;

    if exist_attachment.is_none() {
        return Err(FieldError::InvalidParams("cid".to_string()));
    }

    let exist_attachment = exist_attachment.unwrap();

    let mut c = content::ActiveModel::from(exist_attachment);
    c.title = Set(Some(name.to_owned()));
    c.slug = Set(Some(name.to_owned()));
    c.modified = Set(now);
    c.text = Set(Some(text.to_owned()));
    c.update(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("modify attachment failed".to_string()))
}

pub async fn get_attachments_by_list_query(
    state: &AppState,
    private: bool,
    page_size: u64,
    page: u64,
    order_by: &str,
) -> Result<Vec<content::Model>, FieldError> {
    let stmt = Content::find().filter(content::Column::Type.eq("attachment"));
    let stmt = if private {
        stmt
    } else {
        stmt.filter(content::Column::Status.eq("publish"))
    };
    let stmt = match order_by {
        "-cid" => stmt.order_by_desc(content::Column::Cid),
        "cid" => stmt.order_by_asc(content::Column::Cid),
        "-slug" => stmt.order_by_desc(content::Column::Slug),
        "slug" => stmt.order_by_asc(content::Column::Slug),
        _ => stmt.order_by_desc(content::Column::Cid),
    };

    let paginator = stmt.paginate(&state.conn, page_size);

    let contents = paginator
        .fetch_page(page - 1)
        .await
        .map_err(|_| FieldError::DatabaseFailed("fetch content failed".to_string()))?;
    Ok(contents)
}

pub async fn get_attachments_by_parent(
    state: &AppState,
    parent: u32,
) -> Result<Vec<content::Model>, FieldError> {
    Content::find()
        .filter(content::Column::Parent.eq(parent))
        .all(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("fetch content failed".to_string()))
}

pub async fn modify_attachment_parent_by_cid(
    state: &AppState,
    cid: u32,
    parent: u32,
) -> Result<content::Model, FieldError> {
    let exist_attachment = common_db::get_content_by_cid(state, cid).await?;
    if exist_attachment.is_none() {
        return Err(FieldError::InvalidParams("cid".to_string()));
    }
    let exist_attachment = exist_attachment.unwrap();
    let mut c = content::ActiveModel::from(exist_attachment);
    c.parent = Set(parent);
    c.update(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("modify attachment failed".to_string()))
}
