use std::time::SystemTime;

use sea_orm::*;

use super::forms::PostCreate;
use crate::common::errors::FieldError;
use crate::common::models::ContentWithMetasUsersFields;
use crate::entity::{content, content::Entity as Content, field, meta, relationship, user};
use crate::AppState;

pub async fn create_post_by_post_create_with_uid(
    state: &AppState,
    post_create: &PostCreate,
    uid: u32,
) -> Result<content::ActiveModel, FieldError> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;
    let allow_comment = match post_create.allowComment.unwrap_or(true) {
        true => "1",
        false => "0",
    };
    let allow_ping = match post_create.allowPing.unwrap_or(true) {
        true => "1",
        false => "0",
    };
    let allow_feed = match post_create.allowFeed.unwrap_or(true) {
        true => "1",
        false => "0",
    };

    content::ActiveModel {
        r#type: Set("post".to_string()),
        title: Set(Some(post_create.title.to_owned())),
        slug: Set(Some(post_create.slug.to_owned())),
        created: Set(now),
        modified: Set(now),
        text: Set(Some(post_create.text.to_owned())),
        author_id: Set(uid),
        status: Set(post_create.status.to_owned()),
        password: Set(post_create.password.to_owned()),
        allow_comment: Set(allow_comment.to_string()),
        allow_ping: Set(allow_ping.to_string()),
        allow_feed: Set(allow_feed.to_string()),
        ..Default::default()
    }
    .save(&state.conn)
    .await
    .map_err(|_| FieldError::DatabaseFailed("create post failed".to_string()))
}

pub async fn modify_post_by_post_create_with_exist_post(
    state: &AppState,
    post_modify: &PostCreate,
    exist_post: &content::Model,
) -> Result<content::Model, FieldError> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;
    let now = if now > post_modify.created {
        now
    } else {
        post_modify.created
    };

    let allow_comment = match post_modify
        .allowComment
        .unwrap_or(exist_post.allow_comment == "1")
    {
        true => "1",
        false => "0",
    };
    let allow_ping = match post_modify
        .allowPing
        .unwrap_or(exist_post.allow_ping == "1")
    {
        true => "1",
        false => "0",
    };
    let allow_feed = match post_modify
        .allowFeed
        .unwrap_or(exist_post.allow_feed == "1")
    {
        true => "1",
        false => "0",
    };

    let mut c = content::ActiveModel::from(exist_post.clone());
    c.title = Set(Some(post_modify.title.to_owned()));
    c.slug = Set(Some(post_modify.slug.to_owned()));
    c.created = Set(now);
    c.modified = Set(now);
    c.text = Set(Some(post_modify.text.to_owned()));
    c.status = Set(post_modify.status.to_owned());
    c.password = Set(post_modify.password.to_owned());
    c.allow_comment = Set(allow_comment.to_string());
    c.allow_ping = Set(allow_ping.to_string());
    c.allow_feed = Set(allow_feed.to_string());
    c.update(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("update post failed".to_string()))
}

pub async fn get_contents_with_metas_user_and_fields_by_filter_and_list_query(
    state: &AppState,
    private: bool,
    own: bool,
    author: &user::Model,
    page_size: u64,
    page: u64,
    order_by: &str,
    post: bool,
) -> Result<Vec<ContentWithMetasUsersFields>, FieldError> {
    let content_type = if post { "post" } else { "page" };

    let stmt = Content::find().filter(content::Column::Type.eq(content_type));
    let stmt = if own {
        stmt.filter(content::Column::AuthorId.eq(author.uid))
    } else {
        stmt
    };
    let stmt = if !private {
        stmt.filter(content::Column::Status.eq("publish"))
    } else {
        stmt
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
        .map_err(|_| FieldError::DatabaseFailed("fetch contents failed".to_string()))?;

    let metas = contents
        .load_many_to_many(meta::Entity, relationship::Entity, &state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("fetch metas failed".to_string()))?;

    let fields = contents
        .load_many(field::Entity, &state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("fetch fields failed".to_string()))?;

    let mut res = vec![];
    for ((content, meta_list), field_list) in contents
        .into_iter()
        .zip(metas.into_iter())
        .zip(fields.into_iter())
    {
        let mut ct = ContentWithMetasUsersFields::from(content);
        ct.screen_name = author.screen_name.clone();
        ct.group = author.group.clone();

        let mut tags = vec![];
        let mut categories = vec![];
        for m in meta_list {
            if m.r#type == "tag" {
                tags.push(m);
            } else {
                categories.push(m);
            }
        }
        ct.tags = tags;
        ct.categories = categories;
        ct.fields = field_list;
        res.push(ct);
    }
    Ok(res)
}

pub async fn get_content_with_metas_user_fields_by_slug_and_private(
    state: &AppState,
    slug: &str,
    private: bool,
) -> Result<ContentWithMetasUsersFields, FieldError> {
    let stmt = Content::find().filter(content::Column::Slug.eq(slug));
    let stmt = if private {
        stmt
    } else {
        stmt.filter(content::Column::Status.eq("publish"))
    };

    let content = stmt
        .one(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("fetch content failed".to_string()))?;
    let content = match content {
        Some(c) => c,
        None => return Err(FieldError::InvalidParams("slug".to_string())),
    };

    let metas = content
        .find_related(meta::Entity)
        .all(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("fetch metas failed".to_string()))?;

    let fields = content
        .find_related(field::Entity)
        .all(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("fetch fields failed".to_string()))?;

    let mut res = ContentWithMetasUsersFields::from(content);
    res.tags = metas
        .iter()
        .filter(|m| m.r#type == "tag")
        .cloned()
        .collect();
    res.categories = metas
        .iter()
        .filter(|m| m.r#type == "category")
        .cloned()
        .collect();
    res.fields = fields;
    Ok(res)
}
