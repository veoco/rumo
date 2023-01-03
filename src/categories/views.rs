use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde_json::{json, Value};
use std::sync::Arc;

use super::db;
use super::models::{CategoriesQuery, CategoryCreate, CategoryPostAdd};
use crate::posts::db as post_db;
use crate::posts::models::PostsQuery;
use crate::users::errors::FieldError;
use crate::users::extractors::{PMEditor, PMVisitor, ValidatedJson, ValidatedQuery};
use crate::AppState;

pub async fn create_category(
    State(state): State<Arc<AppState>>,
    PMEditor(_): PMEditor,
    ValidatedJson(category_create): ValidatedJson<CategoryCreate>,
) -> Result<(StatusCode, Json<Value>), FieldError> {
    let exist_cate = db::get_category_by_slug(&state, &category_create.slug).await;
    if exist_cate.is_some() {
        return Err(FieldError::AlreadyExist("slug".to_owned()));
    }

    let row_id = db::create_category_by_category_create(&state, &category_create).await?;
    Ok((StatusCode::CREATED, Json(json!({ "id": row_id }))))
}

pub async fn list_categories(
    State(state): State<Arc<AppState>>,
    ValidatedQuery(q): ValidatedQuery<CategoriesQuery>,
) -> Result<Json<Value>, FieldError> {
    let all_count = db::get_categories_count(&state).await;

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

    let categories = db::get_categories_by_list_query(&state, page_size, offset, order_by).await?;
    Ok(Json(json!({
        "page": page,
        "page_size": page_size,
        "all_count": all_count,
        "count": categories.len(),
        "results": categories
    })))
}

pub async fn get_category_by_slug(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<Value>, FieldError> {
    match db::get_category_by_slug(&state, &slug).await {
        Some(category) => Ok(Json(json!(category))),
        None => Err(FieldError::InvalidParams("slug".to_string())),
    }
}

pub async fn add_post_to_category(
    State(state): State<Arc<AppState>>,
    PMEditor(_): PMEditor,
    Path(slug): Path<String>,
    ValidatedJson(category_post_add): ValidatedJson<CategoryPostAdd>,
) -> Result<(StatusCode, Json<Value>), FieldError> {
    let mid = match db::get_category_by_slug(&state, &slug).await {
        Some(category) => category.mid,
        None => return Err(FieldError::InvalidParams("slug".to_string())),
    };

    let cid = match post_db::get_post_by_slug(&state, &category_post_add.slug).await {
        Some(post) => post.cid,
        None => return Err(FieldError::InvalidParams("post slug".to_string())),
    };

    let exist = post_db::check_relationship_by_cid_and_mid(&state, cid, mid).await?;

    if !exist {
        let _ = post_db::create_relationship_by_cid_and_mid(&state, cid, mid).await?;
        let _ = db::update_category_by_mid_for_count(&state, mid).await?;
        Ok((StatusCode::CREATED, Json(json!({"msg": "ok"}))))
    } else {
        Err(FieldError::AlreadyExist("slug".to_string()))
    }
}

pub async fn list_category_posts_by_slug(
    State(state): State<Arc<AppState>>,
    PMVisitor(user): PMVisitor,
    Path(slug): Path<String>,
    ValidatedQuery(q): ValidatedQuery<PostsQuery>,
) -> Result<Json<Value>, FieldError> {
    let mid = match db::get_category_by_slug(&state, &slug).await {
        Some(category) => category.mid,
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

    let all_count = db::get_category_posts_count_by_mid(&state, mid, &private_sql).await;

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

    let posts = db::get_category_posts_with_meta_by_mid_and_list_query(
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
