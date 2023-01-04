use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde_json::{json, Value};
use std::sync::Arc;

use super::db;
use super::models::{PageCreate, PagesQuery, FieldCreate};
use crate::users::errors::FieldError;
use crate::users::extractors::{PMEditor, PMVisitor, ValidatedJson, ValidatedQuery};
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

pub async fn modify_page_by_slug(
    State(state): State<Arc<AppState>>,
    PMEditor(_): PMEditor,
    Path(slug): Path<String>,
    ValidatedJson(page_modify): ValidatedJson<PageCreate>,
)-> Result<Json<Value>, FieldError>{
    let exist_page = db::get_page_by_slug(&state, &slug).await;
    if exist_page.is_none() {
        return Err(FieldError::NotFound("slug".to_owned()));
    }
    let exist_page = exist_page.unwrap();

    let target_page = db::get_page_by_slug(&state, &page_modify.slug).await;
    if target_page.is_some() {
        return Err(FieldError::AlreadyExist("page slug".to_owned()));
    }

    let row_id = db::modify_page_by_page_modify_with_exist_page(&state, &page_modify, &exist_page).await?;
    Ok(Json(json!({ "id": row_id })))
}

pub async fn list_pages(
    State(state): State<Arc<AppState>>,
    PMVisitor(user): PMVisitor,
    ValidatedQuery(q): ValidatedQuery<PagesQuery>,
) -> Result<Json<Value>, FieldError> {
    let admin = user.group == "editor" || user.group == "administrator";
    let private = q.private.unwrap_or(false);
    if private && !admin {
        return Err(FieldError::PermissionDeny);
    }

    let private_sql = if private {
        String::from("")
    } else {
        format!(
            r#" AND {contents_table}."status" == 'publish'"#,
            contents_table = &state.contents_table,
        )
    };

    let all_count = db::get_pages_count_with_private(&state, &private_sql).await;

    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(10);
    let order_by = q.order_by.unwrap_or("-cid".to_string());

    let offset = (page - 1) * page_size;
    let order_by = match order_by.as_str() {
        "cid" => "cid",
        "-cid" => "cid DESC",
        "slug" => "slug",
        "-slug" => "slug DESC",
        "order" => "order",
        "-order" => "order DESC",
        f => return Err(FieldError::InvalidParams(f.to_string())),
    };

    let pages = db::get_pages_by_list_query_with_private(&state, &private_sql, page_size, offset, order_by).await?;
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
    PMVisitor(user): PMVisitor,
    Path(slug): Path<String>,
) -> Result<Json<Value>, FieldError> {
    let page = db::get_page_with_meta_by_slug(&state, &slug).await?;
    let admin = user.group == "editor" || user.group == "administrator";

    if page.status == "hidden" && !admin{
        Err(FieldError::PermissionDeny)
    }else{
        Ok(Json(json!(page)))
    }
}

pub async fn create_page_field_by_slug(
    State(state): State<Arc<AppState>>,
    PMEditor(_): PMEditor,
    Path(slug): Path<String>,
    ValidatedJson(field_create): ValidatedJson<FieldCreate>,
) -> Result<(StatusCode, Json<Value>), FieldError> {
    let exist_page = db::get_page_by_slug(&state, &slug).await;
    if exist_page.is_none() {
        return Err(FieldError::AlreadyExist("slug".to_owned()));
    }
    let exist_page = exist_page.unwrap();

    let row_id = db::create_field_by_cid_with_field_create(&state, exist_page.cid, &field_create).await?;
    Ok((StatusCode::CREATED, Json(json!({ "id": row_id }))))
}
