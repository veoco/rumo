use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde_json::{json, Value};
use std::sync::Arc;

use super::db;
use super::forms::PageCreate;
use crate::common::db as common_db;
use crate::common::errors::FieldError;
use crate::common::extractors::{PMEditor, PMVisitor, ValidatedJson, ValidatedQuery};
use crate::common::forms::FieldCreate;
use crate::common::forms::ListQueryWithPrivate;
use crate::AppState;

pub async fn create_page(
    State(state): State<Arc<AppState>>,
    PMEditor(user): PMEditor,
    ValidatedJson(page_create): ValidatedJson<PageCreate>,
) -> Result<(StatusCode, Json<Value>), FieldError> {
    match common_db::get_content_by_slug(&state, &page_create.slug).await {
        Ok(Some(_)) => return Err(FieldError::AlreadyExist("page".to_owned())),
        _ => (),
    };

    let _ = db::create_page_by_page_create_with_uid(&state, &page_create, user.uid).await?;
    Ok((StatusCode::CREATED, Json(json!({ "msg": "ok" }))))
}

pub async fn modify_page_by_slug(
    State(state): State<Arc<AppState>>,
    PMEditor(_): PMEditor,
    Path(slug): Path<String>,
    ValidatedJson(page_modify): ValidatedJson<PageCreate>,
) -> Result<Json<Value>, FieldError> {
    let exist_page = match common_db::get_content_by_slug(&state, &slug).await {
        Ok(Some(p)) => p,
        _ => return Err(FieldError::NotFound("page".to_owned())),
    };

    if slug != page_modify.slug {
        match common_db::get_content_by_slug(&state, &page_modify.slug).await {
            Ok(Some(_)) => return Err(FieldError::AlreadyExist("page slug".to_owned())),
            _ => (),
        };
    }

    let _ =
        db::modify_page_by_page_modify_with_exist_page(&state, &page_modify, &exist_page).await?;
    Ok(Json(json!({ "msg": "ok" })))
}

pub async fn list_pages(
    State(state): State<Arc<AppState>>,
    PMVisitor(user): PMVisitor,
    ValidatedQuery(q): ValidatedQuery<ListQueryWithPrivate>,
) -> Result<Json<Value>, FieldError> {
    let admin = user.group == "editor" || user.group == "administrator";
    let private = q.private.unwrap_or(false);
    if private && !admin {
        return Err(FieldError::PermissionDeny);
    }

    let all_count =
        common_db::get_contents_count_with_private(&state, private, false, &user, "page").await;

    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(10);
    let order_by = q.order_by.unwrap_or("-cid".to_string());

    let pages = db::get_contents_with_fields_by_list_query_with_private(
        &state, private, page_size, page, &order_by, false,
    )
    .await?;
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
    let page = db::get_content_with_fields_by_slug(&state, &slug).await?;
    let admin = user.group == "editor" || user.group == "administrator";

    if page.status == "hidden" && !admin {
        Err(FieldError::PermissionDeny)
    } else {
        Ok(Json(json!(page)))
    }
}

pub async fn delete_page_by_slug(
    State(state): State<Arc<AppState>>,
    PMEditor(_): PMEditor,
    Path(slug): Path<String>,
) -> Result<Json<Value>, FieldError> {
    let page = match common_db::get_content_by_slug(&state, &slug).await {
        Ok(Some(p)) => p,
        _ => return Err(FieldError::NotFound("slug".to_owned())),
    };

    let _ = common_db::delete_fields_by_cid(&state, page.cid).await?;

    let _ = common_db::delete_content_by_cid(&state, page.cid).await?;
    Ok(Json(json!({ "msg": "ok" })))
}

pub async fn create_page_field_by_slug(
    State(state): State<Arc<AppState>>,
    PMEditor(_): PMEditor,
    Path(slug): Path<String>,
    ValidatedJson(field_create): ValidatedJson<FieldCreate>,
) -> Result<(StatusCode, Json<Value>), FieldError> {
    let exist_page = match common_db::get_content_by_slug(&state, &slug).await {
        Ok(Some(p)) => p,
        _ => return Err(FieldError::NotFound("slug".to_owned())),
    };

    let _ = common_db::create_field_by_cid_with_field_create(&state, exist_page.cid, &field_create)
        .await?;
    Ok((StatusCode::CREATED, Json(json!({ "msg": "ok" }))))
}

pub async fn get_page_field_by_slug_and_name(
    State(state): State<Arc<AppState>>,
    Path((slug, name)): Path<(String, String)>,
) -> Result<Json<Value>, FieldError> {
    let exist_page = match common_db::get_content_by_slug(&state, &slug).await {
        Ok(Some(p)) => p,
        _ => return Err(FieldError::NotFound("slug".to_owned())),
    };

    let field = match common_db::get_field_by_cid_and_name(&state, exist_page.cid, &name).await {
        Ok(Some(f)) => f,
        _ => return Err(FieldError::NotFound("name".to_owned())),
    };
    Ok(Json(json!(field)))
}

pub async fn delete_page_field_by_slug_and_name(
    State(state): State<Arc<AppState>>,
    PMEditor(_): PMEditor,
    Path((slug, name)): Path<(String, String)>,
) -> Result<Json<Value>, FieldError> {
    let exist_page = match common_db::get_content_by_slug(&state, &slug).await {
        Ok(Some(p)) => p,
        _ => return Err(FieldError::NotFound("slug".to_owned())),
    };

    match common_db::get_field_by_cid_and_name(&state, exist_page.cid, &name).await {
        Ok(Some(f)) => f,
        _ => return Err(FieldError::NotFound("name".to_owned())),
    };

    let _ = common_db::delete_field_by_cid_and_name(&state, exist_page.cid, &name).await?;
    Ok(Json(json!({ "msg": "ok" })))
}

pub async fn modify_page_field_by_slug_and_name(
    State(state): State<Arc<AppState>>,
    PMEditor(_): PMEditor,
    Path((slug, name)): Path<(String, String)>,
    ValidatedJson(field_modfify): ValidatedJson<FieldCreate>,
) -> Result<Json<Value>, FieldError> {
    let exist_page = match common_db::get_content_by_slug(&state, &slug).await {
        Ok(Some(p)) => p,
        _ => return Err(FieldError::NotFound("slug".to_owned())),
    };

    if name != field_modfify.name {
        match common_db::get_field_by_cid_and_name(&state, exist_page.cid, &name).await {
            Ok(Some(f)) => f,
            _ => return Err(FieldError::NotFound("name".to_owned())),
        };
    }

    let _ = common_db::modify_field_by_cid_and_name_with_field_create(
        &state,
        exist_page.cid,
        &name,
        &field_modfify,
    )
    .await?;
    Ok(Json(json!({ "msg": "ok" })))
}
