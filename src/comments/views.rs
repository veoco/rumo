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
use crate::users::extractors::{PMVisitor, PMEditor, ValidatedJson, ValidatedQuery};
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

    let content_sql = format!(
        r#"
        SELECT *
        FROM {contents_table}
        WHERE {contents_table}."slug" == ?1
        "#,
        contents_table = &state.contents_table,
    );
    let content = match sqlx::query_as::<_, Post>(&content_sql)
        .bind(slug)
        .fetch_one(&state.pool)
        .await
    {
        Ok(p) => p,
        Err(_) => return Err(FieldError::InvalidParams("slug".to_string())),
    };

    if content.allowComment == "0" {
        return Err(FieldError::InvalidParams("slug".to_string()));
    }

    let mut parent = 0;
    if let Some(coid) = comment_create.parent {
        let parent_sql = format!(
            r#"
            SELECT *
            FROM {comments_table}
            WHERE {comments_table}."status" == 'approved' AND {comments_table}."type" == 'comment' AND {comments_table}."cid" == ?1 AND {comments_table}."coid" == ?2
            "#,
            comments_table = &state.comments_table
        );
        match sqlx::query_as::<_, Comment>(&parent_sql)
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

    let insert_sql = format!(
        r#"
        INSERT INTO {comments_table} ("type", "cid", "created", "author", "authorId", "ownerId", "mail", "url", "ip", "agent", "text", "status", "parent")
        VALUES ('comment', ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
        "#,
        comments_table = &state.comments_table
    );
    match sqlx::query(&insert_sql)
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
        Ok(r) => {
            return Ok((
                StatusCode::CREATED,
                Json(json!({"id": r.last_insert_rowid()})),
            ))
        }
        Err(e) => return Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn list_comments(
    State(state): State<Arc<AppState>>,
    PMEditor(_): PMEditor,
    ValidatedQuery(q): ValidatedQuery<CommentsQuery>,
) -> Result<Json<Value>, FieldError> {
    let all_sql = format!(
        r#"
        SELECT COUNT(*)
        FROM {comments_table}
        WHERE {comments_table}."type" == 'comment'
        "#,
        comments_table = &state.comments_table
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
        FROM {comments_table}
        WHERE {comments_table}."type" == 'comment'
        ORDER BY {comments_table}.{}
        LIMIT ?1 OFFSET ?2"#,
        order_by,
        comments_table = &state.comments_table
    );
    match sqlx::query_as::<_, Comment>(&sql)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
    {
        Ok(comments) => {
            return Ok(Json(json!({
                "page": page,
                "page_size": page_size,
                "all_count": all_count,
                "count": comments.len(),
                "results": comments
            })));
        }
        Err(e) => return Err(FieldError::DatabaseFailed(e.to_string())),
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
        String::from("")
    } else {
        format!(
            r#" AND {comments_table}."status" == 'approved'"#,
            comments_table = &state.comments_table
        )
    };

    let content_sql = format!(
        r#"
        SELECT *
        FROM {contents_table}
        WHERE {contents_table}."slug" == ?1
        "#,
        contents_table = &state.contents_table
    );
    let content = match sqlx::query_as::<_, Post>(&content_sql)
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
        FROM {comments_table}
        WHERE {comments_table}."type" == 'comment' AND {comments_table}."cid" == ?1{}
        "#,
        private_sql,
        comments_table = &state.comments_table
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
        FROM {comments_table}
        WHERE {comments_table}."type" == 'comment' AND {comments_table}."cid" == ?1{}
        ORDER BY {comments_table}.{}
        LIMIT ?2 OFFSET ?3"#,
        private_sql,
        order_by,
        comments_table = &state.comments_table
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
