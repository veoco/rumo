use std::time::SystemTime;

use sea_orm::*;

use super::forms::PageCreate;
use crate::common::errors::FieldError;
use crate::common::models::ContentWithFields;
use crate::entity::{content, content::Entity as Content, field::Entity as ContentField};
use crate::AppState;

pub async fn create_page_by_page_create_with_uid(
    state: &AppState,
    page_create: &PageCreate,
    uid: u32,
) -> Result<content::ActiveModel, FieldError> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;
    let status = match page_create.publish.unwrap_or(true) {
        true => "publish",
        false => "hidden",
    };
    let allow_comment = match page_create.allowComment.unwrap_or(true) {
        true => "1",
        false => "0",
    };
    let allow_ping = match page_create.allowPing.unwrap_or(true) {
        true => "1",
        false => "0",
    };
    let allow_feed = match page_create.allowFeed.unwrap_or(true) {
        true => "1",
        false => "0",
    };

    content::ActiveModel {
        r#type: Set("page".to_string()),
        title: Set(Some(page_create.title.to_owned())),
        slug: Set(Some(page_create.slug.to_owned())),
        created: Set(now),
        modified: Set(now),
        text: Set(Some(page_create.text.to_owned())),
        author_id: Set(uid),
        status: Set(status.to_owned()),
        allow_comment: Set(allow_comment.to_string()),
        allow_ping: Set(allow_ping.to_string()),
        allow_feed: Set(allow_feed.to_string()),
        ..Default::default()
    }
    .save(&state.conn)
    .await
    .map_err(|_| FieldError::DatabaseFailed("create post failed".to_string()))
}

pub async fn modify_page_by_page_modify_with_exist_page(
    state: &AppState,
    page_modify: &PageCreate,
    exist_page: &content::Model,
) -> Result<content::Model, FieldError> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;
    let now = if now > page_modify.created {
        now
    } else {
        page_modify.created
    };

    let status = match page_modify
        .publish
        .unwrap_or(exist_page.status == "publish")
    {
        true => "publish",
        false => "hidden",
    };
    let allow_comment = match page_modify
        .allowComment
        .unwrap_or(exist_page.allow_comment == "1")
    {
        true => "1",
        false => "0",
    };
    let allow_ping = match page_modify
        .allowPing
        .unwrap_or(exist_page.allow_ping == "1")
    {
        true => "1",
        false => "0",
    };
    let allow_feed = match page_modify
        .allowFeed
        .unwrap_or(exist_page.allow_feed == "1")
    {
        true => "1",
        false => "0",
    };

    let mut c = content::ActiveModel::from(exist_page.clone());
    c.title = Set(Some(page_modify.title.to_owned()));
    c.slug = Set(Some(page_modify.slug.to_owned()));
    c.created = Set(now);
    c.modified = Set(now);
    c.text = Set(Some(page_modify.text.to_owned()));
    c.status = Set(status.to_owned());
    c.allow_comment = Set(allow_comment.to_string());
    c.allow_ping = Set(allow_ping.to_string());
    c.allow_feed = Set(allow_feed.to_string());
    c.update(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("update post failed".to_string()))
}

pub async fn get_content_with_fields_by_slug(
    state: &AppState,
    slug: &str,
) -> Result<ContentWithFields, FieldError> {
    let c = Content::find()
        .filter(content::Column::Slug.eq(slug))
        .one(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("fetch content failed".to_string()))?;
    let c = match c {
        Some(c) => c,
        None => return Err(FieldError::NotFound("slug".to_string())),
    };
    let fields = c
        .find_related(ContentField)
        .all(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("fetch fields failed".to_string()))?;
    let mut res = ContentWithFields::from(c);
    res.fields = fields;
    Ok(res)
}

pub async fn get_contents_with_fields_by_list_query_with_private(
    state: &AppState,
    private: bool,
    page_size: u64,
    page: u64,
    order_by: &str,
    post: bool,
) -> Result<Vec<ContentWithFields>, FieldError> {
    let content_type = if post { "post" } else { "page" };

    let stmt = Content::find().filter(content::Column::Type.eq(content_type));
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
    let fields = contents
        .load_many(ContentField, &state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("fetch fields failed".to_string()))?;

    let mut res = vec![];
    for (content, field_list) in contents.into_iter().zip(fields.into_iter()) {
        let mut ct = ContentWithFields::from(content);
        ct.fields = field_list;
        res.push(ct);
    }
    Ok(res)
}
