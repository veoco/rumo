use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::response::Json;
use chrono::prelude::*;
use rand::Rng;
use serde_json::{json, Value};
use std::sync::Arc;

use super::db;
use super::models::{AttachmentInfo, AttachmentText, AttachmentsQuery};
use super::ser::to_string;
use super::utils::stream_to_file;
use crate::users::errors::FieldError;
use crate::users::extractors::{PMContributor, ValidatedQuery};
use crate::AppState;

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

    let path = format!("usr/uploads/{}/{}/{}", now.year(), now.month(), name);
    let base_dir = std::path::Path::new(&state.upload_root).join(&path);
    let size = stream_to_file(base_dir, &name, field).await?;

    let path = format!("/{path}");
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

    let row_id = db::create_attachment_with_params(
        &state,
        &text.name,
        now_timestamp,
        &attachment_text,
        user.uid,
    )
    .await?;
    let res =
        AttachmentInfo::from_attachment_text(text, row_id as u32, now_timestamp, now_timestamp);
    Ok((StatusCode::CREATED, Json(json!({ "msg": res }))))
}

pub async fn list_attachments(
    State(state): State<Arc<AppState>>,
    PMContributor(user): PMContributor,
    ValidatedQuery(q): ValidatedQuery<AttachmentsQuery>,
) -> Result<Json<Value>, FieldError> {
    let private =
        q.private.unwrap_or(false) && (user.group == "editor" || user.group == "administrator");

    let private_sql = if private {
        String::from("")
    } else {
        format!(
            r#" AND {contents_table}."authorId" == {}"#,
            user.uid,
            contents_table = &state.contents_table,
        )
    };

    let all_count = db::get_attachments_count_with_private(&state, &private_sql).await;

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

    let attachments =
        db::get_attachments_count_by_list_query(&state, &private_sql, page_size, offset, order_by)
            .await?;

    let mut results = vec![];
    for at in attachments {
        let attachment_info = AttachmentInfo::from_attachment(at)
            .map_err(|_| FieldError::InvalidParams("text".to_string()))?;
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
