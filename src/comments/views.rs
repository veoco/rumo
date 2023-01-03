use axum::extract::{Path, State, TypedHeader};
use axum::headers::UserAgent;
use axum::http::StatusCode;
use axum::response::Json;
use axum_client_ip::ClientIp;
use md5::{Digest, Md5};
use serde_json::{json, Value};
use std::sync::Arc;

use super::db;
use super::models::{Comment, CommentCreate, CommentsQuery};
use crate::pages::db as page_db;
use crate::posts::db as post_db;
use crate::users::errors::FieldError;
use crate::users::extractors::{PMEditor, PMVisitor, ValidatedJson, ValidatedQuery};
use crate::AppState;

pub async fn create_page_comment(
    State(state): State<Arc<AppState>>,
    PMVisitor(user): PMVisitor,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    ClientIp(ip): ClientIp,
    Path(slug): Path<String>,
    ValidatedJson(comment_create): ValidatedJson<CommentCreate>,
) -> Result<(StatusCode, Json<Value>), FieldError> {
    let page = match page_db::get_page_by_slug(&state, &slug).await {
        Some(p) => {
            if p.allowComment == "0" {
                return Err(FieldError::InvalidParams("slug".to_string()));
            }
            p
        }
        None => return Err(FieldError::InvalidParams("slug".to_string())),
    };

    let mut parent = 0;
    if let Some(coid) = comment_create.parent {
        if let Some(comment) = db::get_comment_by_coid(&state, coid).await {
            if comment.status == "approved" {
                parent = coid;
            }
        } else {
            return Err(FieldError::InvalidParams("parent".to_string()));
        }
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

    let row_id = db::create_comment_with_params(
        &state,
        page.cid,
        &author,
        author_id,
        page.authorId,
        &mail,
        url,
        &ip,
        &ua,
        &comment_create.text,
        status,
        parent,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(json!({ "id": row_id }))))
}

pub async fn create_post_comment(
    State(state): State<Arc<AppState>>,
    PMVisitor(user): PMVisitor,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    ClientIp(ip): ClientIp,
    Path(slug): Path<String>,
    ValidatedJson(comment_create): ValidatedJson<CommentCreate>,
) -> Result<(StatusCode, Json<Value>), FieldError> {
    let post = match post_db::get_post_by_slug(&state, &slug).await {
        Some(p) => {
            if p.allowComment == "0" {
                return Err(FieldError::InvalidParams("slug".to_string()));
            }
            p
        }
        None => return Err(FieldError::InvalidParams("slug".to_string())),
    };

    let mut parent = 0;
    if let Some(coid) = comment_create.parent {
        if let Some(comment) = db::get_comment_by_coid(&state, coid).await {
            if comment.status == "approved" {
                parent = coid;
            }
        } else {
            return Err(FieldError::InvalidParams("parent".to_string()));
        }
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

    let row_id = db::create_comment_with_params(
        &state,
        post.cid,
        &author,
        author_id,
        post.authorId,
        &mail,
        url,
        &ip,
        &ua,
        &comment_create.text,
        status,
        parent,
    )
    .await?;
    Ok((StatusCode::CREATED, Json(json!({ "id": row_id }))))
}

pub async fn list_comments(
    State(state): State<Arc<AppState>>,
    PMEditor(_): PMEditor,
    ValidatedQuery(q): ValidatedQuery<CommentsQuery>,
) -> Result<Json<Value>, FieldError> {
    let all_count = db::get_comments_count(&state).await;

    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(10);
    let order_by = q.order_by.unwrap_or("-coid".to_string());

    let offset = (page - 1) * page_size;
    let order_by = match order_by.as_str() {
        "coid" => "coid",
        "-coid" => "coid DESC",
        f => return Err(FieldError::InvalidParams(f.to_string())),
    };

    let comments = db::get_comments_by_list_query(&state, page_size, offset, order_by).await?;
    Ok(Json(json!({
        "page": page,
        "page_size": page_size,
        "all_count": all_count,
        "count": comments.len(),
        "results": comments
    })))
}

pub async fn list_page_comments_by_slug(
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

    let target_page = match page_db::get_page_by_slug(&state, &slug).await {
        Some(p) => p,
        None => return Err(FieldError::InvalidParams("slug".to_string())),
    };

    let all_count = db::get_comments_count_with_private(&state, &private_sql).await;

    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(10);
    let order_by = q.order_by.unwrap_or("-coid".to_string());

    let offset = (page - 1) * page_size;
    let order_by = match order_by.as_str() {
        "coid" => "coid",
        "-coid" => "coid DESC",
        f => return Err(FieldError::InvalidParams(f.to_string())),
    };

    let comments = db::get_comments_by_cid_and_list_query_with_private(
        &state,
        target_page.cid,
        &private_sql,
        page_size,
        offset,
        order_by,
    )
    .await?;

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

pub async fn list_post_comments_by_slug(
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

    let target_post = match post_db::get_post_by_slug(&state, &slug).await {
        Some(p) => p,
        None => return Err(FieldError::InvalidParams("slug".to_string())),
    };

    let all_count = db::get_comments_count_with_private(&state, &private_sql).await;

    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(10);
    let order_by = q.order_by.unwrap_or("-coid".to_string());

    let offset = (page - 1) * page_size;
    let order_by = match order_by.as_str() {
        "coid" => "coid",
        "-coid" => "coid DESC",
        f => return Err(FieldError::InvalidParams(f.to_string())),
    };

    let comments = db::get_comments_by_cid_and_list_query_with_private(
        &state,
        target_post.cid,
        &private_sql,
        page_size,
        offset,
        order_by,
    )
    .await?;

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
