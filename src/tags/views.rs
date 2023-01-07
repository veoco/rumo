use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde_json::{json, Value};
use std::sync::Arc;

use super::db::{self};
use super::models::{TagCreate, TagPostAdd, TagsQuery};
use crate::categories::db as category_db;
use crate::common::errors::FieldError;
use crate::common::extractors::{PMEditor, PMVisitor, ValidatedJson, ValidatedQuery};
use crate::posts::db as post_db;
use crate::posts::models::PostsQuery;
use crate::AppState;

pub async fn create_tag(
    State(state): State<Arc<AppState>>,
    PMEditor(_): PMEditor,
    ValidatedJson(tag_create): ValidatedJson<TagCreate>,
) -> Result<(StatusCode, Json<Value>), FieldError> {
    let exist_tag = db::get_tag_by_slug(&state, &tag_create.slug).await;
    if exist_tag.is_some() {
        return Err(FieldError::AlreadyExist("slug".to_owned()));
    }

    let row_id = db::create_tag_by_tag_create(&state, &tag_create).await?;
    Ok((StatusCode::CREATED, Json(json!({ "id": row_id }))))
}

pub async fn list_tags(
    State(state): State<Arc<AppState>>,
    ValidatedQuery(q): ValidatedQuery<TagsQuery>,
) -> Result<Json<Value>, FieldError> {
    let all_count = db::get_tags_count(&state).await;

    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(10);
    let order_by = q.order_by.unwrap_or("-mid".to_string());

    let offset = (page - 1) * page_size;
    let order_by = match order_by.as_str() {
        "mid" => "mid",
        "-mid" => "mid DESC",
        "slug" => "slug",
        "-slug" => "slug DESC",
        f => return Err(FieldError::InvalidParams(f.to_string())),
    };

    let tags = db::get_tags_by_list_query(&state, page_size, offset, order_by).await?;
    Ok(Json(json!({
        "page": page,
        "page_size": page_size,
        "all_count": all_count,
        "count": tags.len(),
        "results": tags
    })))
}

pub async fn get_tag_by_slug(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<Value>, FieldError> {
    match db::get_tag_by_slug(&state, &slug).await {
        Some(tag) => Ok(Json(json!(tag))),
        None => Err(FieldError::NotFound("slug".to_string())),
    }
}

pub async fn modify_tag_by_slug(
    State(state): State<Arc<AppState>>,
    PMEditor(_): PMEditor,
    Path(slug): Path<String>,
    ValidatedJson(tag_modify): ValidatedJson<TagCreate>,
) -> Result<Json<Value>, FieldError> {
    let exist_tag = db::get_tag_by_slug(&state, &slug).await;
    if exist_tag.is_none() {
        return Err(FieldError::InvalidParams("slug".to_owned()));
    }
    let exist_tag = exist_tag.unwrap();

    let target_tag = db::get_tag_by_slug(&state, &tag_modify.slug).await;
    if target_tag.is_some() {
        return Err(FieldError::InvalidParams("category slug".to_owned()));
    }

    let row_id = db::modify_tag_by_mid_and_tag_modify(&state, exist_tag.mid, &tag_modify).await?;
    Ok(Json(json!({ "id": row_id })))
}

pub async fn delete_tag_by_slug(
    State(state): State<Arc<AppState>>,
    PMEditor(_): PMEditor,
    Path(slug): Path<String>,
) -> Result<Json<Value>, FieldError> {
    let exist_tag = db::get_tag_by_slug(&state, &slug).await;
    if exist_tag.is_none() {
        return Err(FieldError::InvalidParams("slug".to_owned()));
    }
    let exist_tag = exist_tag.unwrap();

    let _ = category_db::delete_relationships_by_mid(&state, exist_tag.mid).await?;
    let _ = category_db::delete_meta_by_mid(&state, exist_tag.mid).await?;
    Ok(Json(json!({ "msg": "ok" })))
}

pub async fn add_post_to_tag(
    State(state): State<Arc<AppState>>,
    PMEditor(_): PMEditor,
    Path(slug): Path<String>,
    ValidatedJson(tag_post_add): ValidatedJson<TagPostAdd>,
) -> Result<(StatusCode, Json<Value>), FieldError> {
    let mid = match db::get_tag_by_slug(&state, &slug).await {
        Some(tag) => tag.mid,
        None => return Err(FieldError::InvalidParams("slug".to_string())),
    };

    let cid = match post_db::get_post_by_slug(&state, &tag_post_add.slug).await {
        Some(post) => post.cid,
        None => return Err(FieldError::InvalidParams("post slug".to_string())),
    };

    let exist = post_db::check_relationship_by_cid_and_mid(&state, cid, mid).await?;

    if !exist {
        let _ = post_db::create_relationship_by_cid_and_mid(&state, cid, mid).await?;
        let _ = db::update_tag_by_mid_for_count(&state, mid).await?;
        Ok((StatusCode::CREATED, Json(json!({"msg": "ok"}))))
    } else {
        Err(FieldError::AlreadyExist("slug".to_string()))
    }
}

pub async fn delete_post_from_tag(
    State(state): State<Arc<AppState>>,
    PMEditor(_): PMEditor,
    Path((slug, post_slug)): Path<(String, String)>,
) -> Result<Json<Value>, FieldError> {
    let mid = match db::get_tag_by_slug(&state, &slug).await {
        Some(tag) => tag.mid,
        None => return Err(FieldError::InvalidParams("slug".to_string())),
    };

    let cid = match post_db::get_post_by_slug(&state, &post_slug).await {
        Some(post) => post.cid,
        None => return Err(FieldError::InvalidParams("post slug".to_string())),
    };

    let exist = post_db::check_relationship_by_cid_and_mid(&state, cid, mid).await?;

    if exist {
        let _ = post_db::delete_relationship_by_cid_and_mid(&state, cid, mid).await?;
        let _ = category_db::update_meta_by_mid_for_decrease_count(&state, mid).await?;
        Ok(Json(json!({"msg": "ok"})))
    } else {
        Err(FieldError::AlreadyExist("slug".to_string()))
    }
}

pub async fn list_tag_posts_by_slug(
    State(state): State<Arc<AppState>>,
    PMVisitor(user): PMVisitor,
    Path(slug): Path<String>,
    ValidatedQuery(q): ValidatedQuery<PostsQuery>,
) -> Result<Json<Value>, FieldError> {
    let mid = match db::get_tag_by_slug(&state, &slug).await {
        Some(tag) => tag.mid,
        None => return Err(FieldError::InvalidParams("slug".to_string())),
    };

    let private =
        q.private.unwrap_or(false) && (user.group == "editor" || user.group == "administrator");
    let private_sql = if private {
        String::from("")
    } else {
        format!(
            r#" AND {contents_table}."status" == 'publish' AND {contents_table}."password" IS NULL"#,
            contents_table = &state.contents_table
        )
    };

    let all_count = db::get_tag_posts_count_by_mid(&state, mid, &private_sql).await;

    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(10);
    let order_by = q.order_by.unwrap_or("-cid".to_string());

    let offset = (page - 1) * page_size;
    let order_by = match order_by.as_str() {
        "cid" => "cid",
        "-cid" => "cid DESC",
        "slug" => "slug",
        "-slug" => "slug DESC",
        f => return Err(FieldError::InvalidParams(f.to_string())),
    };

    let posts = db::get_tag_posts_with_meta_by_mid_and_list_query(
        &state,
        mid,
        &private_sql,
        page_size,
        offset,
        order_by,
    )
    .await?;
    Ok(Json(json!({
        "page": page,
        "page_size": page_size,
        "all_count": all_count,
        "count": posts.len(),
        "results": posts
    })))
}
