use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde_json::{json, Value};
use std::sync::Arc;

use super::db;
use super::models::{PageCreate, PagesQuery};
use crate::users::errors::FieldError;
use crate::users::extractors::{PMEditor, ValidatedJson, ValidatedQuery};
use crate::AppState;

pub async fn create_page(
    State(state): State<Arc<AppState>>,
    PMEditor(user): PMEditor,
    ValidatedJson(page_create): ValidatedJson<PageCreate>,
) -> Result<(StatusCode, Json<Value>), FieldError> {
    let exist_page = db::get_page_by_slug(&state, &page_create.slug).await;
    if exist_page.is_some() {
        return Err(FieldError::AlreadyExist("slug".to_owned()));
    }

    let row_id = db::create_page_by_page_create_with_uid(&state, &page_create, user.uid).await?;
    Ok((StatusCode::CREATED, Json(json!({ "id": row_id }))))
}

pub async fn list_pages(
    State(state): State<Arc<AppState>>,
    ValidatedQuery(q): ValidatedQuery<PagesQuery>,
) -> Result<Json<Value>, FieldError> {
    let all_count = db::get_pages_count(&state).await;

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

    let pages = db::get_pages_by_list_query(&state, page_size, offset, order_by).await?;
    Ok(Json(json!({
        "page": page,
        "page_size": page_size,
        "all_count": all_count,
        "count": pages.len(),
        "results": pages
    })))
}

pub async fn get_page_by_slug(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<Value>, FieldError> {
    let page = db::get_page_with_meta_by_slug(&state, &slug).await?;
    Ok(Json(json!(page)))
}
