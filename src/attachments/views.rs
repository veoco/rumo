use std::sync::Arc;

use axum::extract::{Multipart, Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use chrono::prelude::*;
use rand::Rng;
use serde_json::{json, Value};

use super::db;
use super::de::from_str;
use super::forms::{AttachmentCreate, AttachmentsQuery};
use super::models::{AttachmentInfo, AttachmentText};
use super::ser::to_string;
use super::utils::{delete_file, stream_to_file};
use crate::common::db as common_db;
use crate::common::errors::FieldError;
use crate::common::extractors::{PMContributor, ValidatedJson, ValidatedQuery};
use crate::AppState;

pub async fn list_attachments(
    State(state): State<Arc<AppState>>,
    PMContributor(user): PMContributor,
    ValidatedQuery(q): ValidatedQuery<AttachmentsQuery>,
) -> Result<Json<Value>, FieldError> {
    let private =
        q.private.unwrap_or(false) && (user.group == "editor" || user.group == "administrator");

    let all_count =
        common_db::get_contents_count_with_private(&state, private, false, &user, "attachment")
            .await;

    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(10);
    let order_by = q.order_by.unwrap_or("-cid".to_string());

    let attachments =
        db::get_attachments_by_list_query(&state, private, page_size, page, &order_by).await?;

    let mut results = vec![];
    for at in attachments {
        let attachment_info = AttachmentInfo::from(at);
        results.push(attachment_info);
    }

    Ok(Json(json!({
        "page": page,
        "page_size": page_size,
        "all_count": all_count,
        "count": results.len(),
        "results": results
    })))
}

pub async fn create_attachment(
    State(state): State<Arc<AppState>>,
    PMContributor(user): PMContributor,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<Value>), FieldError> {
    let now = Local::now();
    let field = match multipart.next_field().await {
        Ok(Some(f)) => f,
        _ => return Err(FieldError::InvalidParams("file".to_string())),
    };
    let file_name = match field.file_name() {
        Some(f) => f.to_string(),
        None => return Err(FieldError::InvalidParams("file".to_string())),
    };
    let content_type = match field.content_type() {
        Some(f) => f.to_string(),
        None => return Err(FieldError::InvalidParams("file".to_string())),
    };
    let dot_pos = match file_name.find(".") {
        Some(f) => f,
        None => return Err(FieldError::InvalidParams("file".to_string())),
    };
    let ext = (&file_name[dot_pos + 1..]).to_string();

    let rand_name: u64 = rand::thread_rng().gen_range(1_000_000_000..9_999_999_999);
    let name = format!("{rand_name}.{ext}");

    let filedir = format!("usr/uploads/{}/{}", now.year(), now.month());
    let base_dir = std::path::Path::new(&state.upload_root).join(&filedir);
    let size = stream_to_file(base_dir, &name, field).await?;

    let path = format!("/{filedir}/{name}");
    let text = AttachmentText {
        name: file_name,
        path,
        size,
        r#type: ext,
        mime: content_type,
    };
    let attachment_text = match to_string(&text) {
        Ok(t) => t,
        Err(_) => return Err(FieldError::InvalidParams("file".to_string())),
    };
    let now_timestamp = now.timestamp() as u32;

    let _ = db::create_attachment_with_params(
        &state,
        &text.name,
        now_timestamp,
        &attachment_text,
        user.uid,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(json!({"msg":"ok"}))))
}

pub async fn get_attachment_by_cid(
    State(state): State<Arc<AppState>>,
    PMContributor(user): PMContributor,
    Path(cid): Path<u32>,
) -> Result<Json<Value>, FieldError> {
    let attachment = match common_db::get_content_by_cid(&state, cid).await {
        Ok(Some(a)) => a,
        _ => return Err(FieldError::NotFound("cid".to_string())),
    };

    let admin = user.group == "editor" || user.group == "administrator";
    if user.uid != attachment.author_id && !admin {
        return Err(FieldError::PermissionDeny);
    }

    let at = AttachmentInfo::from(attachment);
    Ok(Json(json!(at)))
}

pub async fn modify_attachment_by_cid(
    State(state): State<Arc<AppState>>,
    PMContributor(user): PMContributor,
    Path(cid): Path<u32>,
    mut multipart: Multipart,
) -> Result<Json<Value>, FieldError> {
    let exist_attachment = match common_db::get_content_by_cid(&state, cid).await {
        Ok(Some(a)) => a,
        _ => return Err(FieldError::NotFound("cid".to_string())),
    };

    let admin = user.group == "editor" || user.group == "administrator";
    if user.uid != exist_attachment.author_id && !admin {
        return Err(FieldError::PermissionDeny);
    }

    let base_dir = std::path::Path::new(&state.upload_root);
    let text = exist_attachment.text.unwrap_or("".to_string());
    let exist_at = from_str::<AttachmentText>(&text)
        .map_err(|_| FieldError::DatabaseFailed("attachment decode error".to_string()))?;
    let _ = delete_file(base_dir.to_path_buf(), &exist_at.path).await;

    let now = Local::now();
    let field = match multipart.next_field().await {
        Ok(Some(f)) => f,
        _ => return Err(FieldError::InvalidParams("file".to_string())),
    };
    let file_name = match field.file_name() {
        Some(f) => f.to_string(),
        None => return Err(FieldError::InvalidParams("file".to_string())),
    };
    let content_type = match field.content_type() {
        Some(f) => f.to_string(),
        None => return Err(FieldError::InvalidParams("file".to_string())),
    };
    let dot_pos = match file_name.find(".") {
        Some(f) => f,
        None => return Err(FieldError::InvalidParams("file".to_string())),
    };
    let ext = (&file_name[dot_pos + 1..]).to_string();

    let rand_name: u64 = rand::thread_rng().gen_range(1_000_000_000..9_999_999_999);
    let name = format!("{rand_name}.{ext}");

    let filedir = format!("usr/uploads/{}/{}", now.year(), now.month());
    let base_dir = std::path::Path::new(&state.upload_root).join(&filedir);
    let size = stream_to_file(base_dir, &name, field).await?;

    let path = format!("/{filedir}/{name}");
    let text = AttachmentText {
        name: file_name,
        path,
        size,
        r#type: ext,
        mime: content_type,
    };
    let attachment_text = match to_string(&text) {
        Ok(t) => t,
        Err(_) => return Err(FieldError::InvalidParams("file".to_string())),
    };
    let now_timestamp = now.timestamp() as u32;

    let _ = db::modify_attachment_by_cid_with_params(
        &state,
        exist_attachment.cid,
        &text.name,
        now_timestamp,
        &attachment_text,
    )
    .await?;
    Ok(Json(json!({"msg":"ok"})))
}

pub async fn delete_attachment_by_cid(
    State(state): State<Arc<AppState>>,
    PMContributor(user): PMContributor,
    Path(cid): Path<u32>,
) -> Result<Json<Value>, FieldError> {
    let attachment = match common_db::get_content_by_cid(&state, cid).await {
        Ok(Some(a)) => a,
        _ => return Err(FieldError::NotFound("cid".to_string())),
    };

    let admin = user.group == "editor" || user.group == "administrator";
    if user.uid != attachment.author_id && !admin {
        return Err(FieldError::PermissionDeny);
    }

    let text = attachment.text.unwrap_or("".to_string());
    let text = from_str::<AttachmentText>(&text)
        .map_err(|_| FieldError::DatabaseFailed("attachment decode error".to_string()))?;

    let base_dir = std::path::Path::new(&state.upload_root);
    let filepath = text.path;
    let _ = delete_file(base_dir.to_path_buf(), &filepath).await;

    let _ = common_db::delete_content_by_cid(&state, cid).await?;
    Ok(Json(json!({ "msg": "ok" })))
}

pub async fn list_content_attachments_by_slug(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<Value>, FieldError> {
    let content = match common_db::get_content_by_slug(&state, &slug).await {
        Ok(Some(c)) => c,
        _ => return Err(FieldError::InvalidParams("slug".to_string())),
    };

    let attachments = db::get_attachments_by_parent(&state, content.cid).await?;

    let mut results = vec![];
    for at in attachments {
        let attachment_info = AttachmentInfo::from(at);
        results.push(attachment_info);
    }

    Ok(Json(json!({
        "page": 1,
        "page_size": results.len(),
        "all_count": results.len(),
        "count": results.len(),
        "results": results
    })))
}

pub async fn add_attachment_to_content_by_cid(
    State(state): State<Arc<AppState>>,
    PMContributor(user): PMContributor,
    Path(slug): Path<String>,
    ValidatedJson(attachement_create): ValidatedJson<AttachmentCreate>,
) -> Result<Json<Value>, FieldError> {
    let attachment = match common_db::get_content_by_cid(&state, attachement_create.cid).await {
        Ok(Some(a)) => a,
        _ => return Err(FieldError::NotFound("cid".to_string())),
    };

    let admin = user.group == "editor" || user.group == "administrator";

    let content = match common_db::get_content_by_slug(&state, &slug).await {
        Ok(Some(c)) => c,
        _ => return Err(FieldError::InvalidParams("slug".to_string())),
    };

    if user.uid != content.author_id && !admin {
        return Err(FieldError::PermissionDeny);
    }

    let _ = db::modify_attachment_parent_by_cid(&state, attachment.cid, content.cid).await?;
    Ok(Json(json!({ "msg": "ok" })))
}

pub async fn delete_attachment_from_content_by_cid(
    State(state): State<Arc<AppState>>,
    PMContributor(user): PMContributor,
    Path((slug, cid)): Path<(String, u32)>,
) -> Result<Json<Value>, FieldError> {
    let attachment = match common_db::get_content_by_cid(&state, cid).await {
        Ok(Some(a)) => a,
        _ => return Err(FieldError::NotFound("cid".to_string())),
    };

    let admin = user.group == "editor" || user.group == "administrator";
    if user.uid != attachment.author_id && !admin {
        return Err(FieldError::PermissionDeny);
    }

    let content = match common_db::get_content_by_slug(&state, &slug).await {
        Ok(Some(c)) => c,
        _ => return Err(FieldError::InvalidParams("slug".to_string())),
    };

    if user.uid != content.author_id && !admin {
        return Err(FieldError::PermissionDeny);
    }

    let _ = db::modify_attachment_parent_by_cid(&state, attachment.cid, 0).await?;
    Ok(Json(json!({ "msg": "ok" })))
}
