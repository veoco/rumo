use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::response::Json;
use chrono::prelude::*;
use rand::Rng;
use serde_json::{json, Value};
use std::sync::Arc;

use super::models::{Attachment, AttachmentInfo, AttachmentText, AttachmentsQuery};
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

    let insert_sql = format!(
        r#"
        INSERT INTO {contents_table} ("type", "title", "slug", "created", "modified", "text", "authorId")
        VALUES ('attachment', ?1, ?1, ?2, ?2, ?3, ?4)
        "#,
        contents_table = &state.contents_table,
    );
    if let Ok(r) = sqlx::query(&insert_sql)
        .bind(&text.name)
        .bind(now_timestamp)
        .bind(attachment_text)
        .bind(user.uid)
        .execute(&state.pool)
        .await
    {
        let cid = r.last_insert_rowid();
        let res =
            AttachmentInfo::from_attachment_text(text, cid as u32, now_timestamp, now_timestamp);
        return Ok((StatusCode::CREATED, Json(json!({ "msg": res }))));
    }

    Err(FieldError::InvalidParams("file".to_string()))
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

    let all_sql = format!(
        r#"
        SELECT COUNT(*)
        FROM {contents_table}
        WHERE {contents_table}.type == 'attachment'{}
        "#,
        private_sql,
        contents_table = &state.contents_table,
    );
    let all_count = sqlx::query_scalar::<_, i32>(&all_sql)
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);

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

    let sql = format!(
        r#"
        SELECT *
        FROM {contents_table}
        WHERE {contents_table}."type" == 'attachment'{}
        ORDER BY {}
        LIMIT ?1 OFFSET ?2"#,
        private_sql,
        order_by,
        contents_table = &state.contents_table,
    );
    match sqlx::query_as::<_, Attachment>(&sql)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
    {
        Ok(ats) => {
            let mut results = vec![];
            for at in ats {
                let attachment_info = AttachmentInfo::from_attachment(at)
                    .map_err(|_| FieldError::InvalidParams("text".to_string()))?;
                results.push(attachment_info);
            }
            return Ok(Json(json!({
                "page": page,
                "page_size": page_size,
                "all_count": all_count,
                "count": results.len(),
                "results": results
            })));
        }
        Err(e) => return Err(FieldError::DatabaseFailed(e.to_string())),
    }
}
