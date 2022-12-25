use axum::extract::{Path, State, TypedHeader};
use axum::headers::UserAgent;
use axum::http::StatusCode;
use axum::response::Json;
use axum_client_ip::ClientIp;
use md5::{Digest, Md5};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::SystemTime;

use super::models::{Comment, CommentCreate, CommentsQuery};
use crate::posts::models::Post;
use crate::users::errors::FieldError;
use crate::users::extractors::{PMVisitor, ValidatedJson, ValidatedQuery};
use crate::AppState;

pub async fn create_comment(
    State(state): State<Arc<AppState>>,
    PMVisitor(user): PMVisitor,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    ClientIp(ip): ClientIp,
    Path(slug): Path<String>,
    ValidatedJson(comment_create): ValidatedJson<CommentCreate>,
) -> Result<(StatusCode, Json<Value>), FieldError> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let content = match sqlx::query_as::<_, Post>(
        r#"
        SELECT *
        FROM typecho_contents
        WHERE slug == ?1"#,
    )
    .bind(slug)
    .fetch_one(&state.pool)
    .await
    {
        Ok(p) => p,
        Err(_) => return Err(FieldError::InvalidParams("slug".to_string())),
    };

    let mut parent = 0;
    if let Some(coid) = comment_create.parent {
        match sqlx::query_as::<_, Comment>(
            r#"
            SELECT *
            FROM typecho_comments
            WHERE status == "approved" AND type == "comment" AND cid == ?1 AND coid == ?2"#,
        )
        .bind(content.cid)
        .bind(coid)
        .fetch_one(&state.pool)
        .await
        {
            Ok(_) => {}
            Err(_) => return Err(FieldError::InvalidParams("slug".to_string())),
        };
        parent = coid;
    }

    if user.group == "visitor" && (comment_create.author.is_none() || comment_create.mail.is_none())
    {
        return Err(FieldError::InvalidParams("author or mail".to_string()));
    }

    let author;
    let author_id;
    let mail;
    let url;
    match user.group.as_str() {
        "visitor" => {
            author = comment_create.author.unwrap();
            author_id = 0;
            mail = comment_create.mail.unwrap();
            url = comment_create.url;
        }
        _ => {
            author = user.screenName.unwrap_or("".to_string());
            author_id = user.uid;
            mail = user.mail.unwrap_or("".to_string());
            url = user.url;
        }
    };
    let ip = ip.to_string();
    let ua = user_agent.to_string();
    let status = "approved";

    match sqlx::query(
      r#"
        INSERT INTO typecho_comments (type, cid, created, author, authorId, ownerId, mail, url, ip, agent, text, status, parent)
        VALUES ("comment", ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)"#,
    )
    .bind(content.cid)
    .bind(now as u32)
    .bind(author)
    .bind(author_id)
    .bind(content.authorId)
    .bind(mail)
    .bind(url)
    .bind(ip)
    .bind(ua)
    .bind(comment_create.text)
    .bind(status)
    .bind(parent)
    .execute(&state.pool)
    .await
    {
        Ok(r) => return Ok((StatusCode::CREATED,Json(json!({"id": r.last_insert_rowid()})))),
        Err(e) => return Err(FieldError::DatabaseFailed(e.to_string()))
    }
}

pub async fn list_content_comments_by_slug(
    State(state): State<Arc<AppState>>,
    PMVisitor(user): PMVisitor,
    Path(slug): Path<String>,
    ValidatedQuery(q): ValidatedQuery<CommentsQuery>,
) -> Result<Json<Value>, FieldError> {
    let private =
        q.private.unwrap_or(false) && (user.group == "editor" || user.group == "administrator");
    let private_sql = if private {
        ""
    } else {
        r#" AND typecho_comments.status == "approved""#
    };

    let content = match sqlx::query_as::<_, Post>(
        r#"
        SELECT *
        FROM typecho_contents
        WHERE slug == ?1"#,
    )
    .bind(slug)
    .fetch_one(&state.pool)
    .await
    {
        Ok(p) => p,
        Err(_) => return Err(FieldError::InvalidParams("slug".to_string())),
    };

    let all_sql = format!(
        r#"
        SELECT COUNT(*)
        FROM typecho_comments
        WHERE type == "comment" AND cid == ?1{}
        "#,
        private_sql
    );
    let all_count = sqlx::query_scalar::<_, i32>(&all_sql)
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);

    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(10);
    let order_by = q.order_by.unwrap_or("-coid".to_string());

    let offset = (page - 1) * page_size;
    let order_by = match order_by.as_str() {
        "coid" => "coid",
        "-coid" => "coid DESC",
        f => return Err(FieldError::InvalidParams(f.to_string())),
    };

    let sql = format!(
        r#"            
        SELECT *
        FROM typecho_comments
        WHERE type == "comment" AND cid == ?1{}
        ORDER BY typecho_comments.{}
        LIMIT ?2 OFFSET ?3"#,
        private_sql, order_by
    );

    match sqlx::query_as::<_, Comment>(&sql)
        .bind(content.cid)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
    {
        Ok(comments) => {
            let mut hasher = Md5::new();
            let mut hashed_comments = vec![];
            for comment in comments {
                let mail = match comment.mail {
                    Some(m) => {
                        hasher.update(m.as_bytes());
                        Some(format!("{:x}", hasher.finalize_reset()))
                    }
                    None => None,
                };
                hashed_comments.push(Comment {
                    mail: mail,
                    ..comment
                })
            }
            return Ok(Json(json!({
                "page": page,
                "page_size": page_size,
                "all_count": all_count,
                "count": hashed_comments.len(),
                "results": hashed_comments
            })));
        }
        Err(e) => return Err(FieldError::DatabaseFailed(e.to_string())),
    }
}
