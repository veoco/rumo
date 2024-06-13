use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use axum_client_ip::InsecureClientIp;
use axum_extra::{headers::UserAgent, TypedHeader};
use md5::{Digest, Md5};
use serde_json::{json, Value};
use std::sync::Arc;

use super::db;
use super::forms::{CommentCreate, CommentModify, CommentsQuery};
use crate::common::db as common_db;
use crate::common::errors::FieldError;
use crate::common::extractors::{PMEditor, PMVisitor, ValidatedJson, ValidatedQuery};
use crate::entity::comment;
use crate::AppState;

pub async fn create_page_comment(
    State(state): State<Arc<AppState>>,
    PMVisitor(user): PMVisitor,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    InsecureClientIp(ip): InsecureClientIp,
    Path(slug): Path<String>,
    ValidatedJson(comment_create): ValidatedJson<CommentCreate>,
) -> Result<(StatusCode, Json<Value>), FieldError> {
    let page = match common_db::get_content_by_slug(&state, &slug).await {
        Ok(Some(p)) => {
            if p.allow_comment == "0" {
                return Err(FieldError::InvalidParams("slug".to_string()));
            }
            p
        }
        _ => return Err(FieldError::InvalidParams("slug".to_string())),
    };

    let mut parent = 0;
    if let Some(coid) = comment_create.parent {
        if let Ok(Some(comment)) = db::get_comment_by_coid(&state, coid).await {
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
            author = user.screen_name.unwrap_or("".to_string());
            author_id = user.uid;
            mail = user.mail.unwrap_or("".to_string());
            url = user.url;
        }
    };
    let ip = ip.to_string();
    let ua = user_agent.to_string();
    let status = "approved";

    let _ = db::create_comment_with_params(
        &state,
        page.cid,
        &author,
        author_id,
        page.author_id,
        &mail,
        url,
        &ip,
        &ua,
        &comment_create.text,
        status,
        parent,
    )
    .await?;
    let _ = db::update_content_count_increase_by_cid(&state, page.cid).await?;
    Ok((StatusCode::CREATED, Json(json!({ "msg": "ok" }))))
}

pub async fn create_post_comment(
    State(state): State<Arc<AppState>>,
    PMVisitor(user): PMVisitor,
    TypedHeader(user_agent): TypedHeader<UserAgent>,
    InsecureClientIp(ip): InsecureClientIp,
    Path(slug): Path<String>,
    ValidatedJson(comment_create): ValidatedJson<CommentCreate>,
) -> Result<(StatusCode, Json<Value>), FieldError> {
    let post = match common_db::get_content_by_slug(&state, &slug).await {
        Ok(Some(p)) => {
            if p.allow_comment == "0" {
                return Err(FieldError::InvalidParams("slug".to_string()));
            }
            p
        }
        _ => return Err(FieldError::InvalidParams("slug".to_string())),
    };

    let mut parent = 0;
    if let Some(coid) = comment_create.parent {
        if let Ok(Some(comment)) = db::get_comment_by_coid(&state, coid).await {
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
            author = user.screen_name.unwrap_or("".to_string());
            author_id = user.uid;
            mail = user.mail.unwrap_or("".to_string());
            url = user.url;
        }
    };
    let ip = ip.to_string();
    let ua = user_agent.to_string();
    let status = "approved";

    let _ = db::create_comment_with_params(
        &state,
        post.cid,
        &author,
        author_id,
        post.author_id,
        &mail,
        url,
        &ip,
        &ua,
        &comment_create.text,
        status,
        parent,
    )
    .await?;
    let _ = db::update_content_count_increase_by_cid(&state, post.cid).await?;
    Ok((StatusCode::CREATED, Json(json!({ "msg": "ok" }))))
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

    let comments = db::get_comments_by_list_query(&state, page_size, page, &order_by).await?;
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

    let target_page = match common_db::get_content_by_slug(&state, &slug).await {
        Ok(Some(p)) => p,
        _ => return Err(FieldError::InvalidParams("slug".to_string())),
    };

    let all_count =
        db::get_content_comments_count_by_cid_with_private(&state, target_page.cid, private).await;

    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(10);
    let order_by = q.order_by.unwrap_or("-coid".to_string());

    let comments = db::get_comments_by_cid_and_list_query_with_private(
        &state,
        target_page.cid,
        private,
        page_size,
        page,
        &order_by,
    )
    .await?;

    let mut hasher = Md5::new();
    let mut hashed_comments = vec![];
    for cm in comments {
        let mail = match cm.mail {
            Some(m) => {
                hasher.update(m.as_bytes());
                Some(format!("{:x}", hasher.finalize_reset()))
            }
            None => None,
        };
        hashed_comments.push(comment::Model { mail: mail, ..cm })
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

    let target_post = match common_db::get_content_by_slug(&state, &slug).await {
        Ok(Some(p)) => p,
        _ => return Err(FieldError::InvalidParams("slug".to_string())),
    };

    let all_count =
        db::get_content_comments_count_by_cid_with_private(&state, target_post.cid, private).await;

    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(10);
    let order_by = q.order_by.unwrap_or("-coid".to_string());

    let comments = db::get_comments_by_cid_and_list_query_with_private(
        &state,
        target_post.cid,
        private,
        page_size,
        page,
        &order_by,
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
        hashed_comments.push(comment::Model {
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

pub async fn get_comment_by_coid(
    State(state): State<Arc<AppState>>,
    PMEditor(_): PMEditor,
    Path(coid): Path<u32>,
) -> Result<Json<Value>, FieldError> {
    match db::get_comment_by_coid(&state, coid).await {
        Ok(Some(comment)) => Ok(Json(json!(comment))),
        _ => Err(FieldError::NotFound("coid".to_string())),
    }
}

pub async fn modify_comment_by_coid(
    State(state): State<Arc<AppState>>,
    PMEditor(_): PMEditor,
    Path(coid): Path<u32>,
    ValidatedJson(comment_modify): ValidatedJson<CommentModify>,
) -> Result<Json<Value>, FieldError> {
    match db::get_comment_by_coid(&state, coid).await {
        Ok(Some(comment)) => Some(comment),
        _ => return Err(FieldError::NotFound("coid".to_string())),
    };

    let status = match comment_modify.status.as_str() {
        "approved" => "approved",
        "waiting" => "waiting",
        "spam" => "spam",
        _ => return Err(FieldError::InvalidParams("status".to_string())),
    };

    let _ = db::modify_comment_with_params(&state, coid, &comment_modify.text, &status).await?;
    Ok(Json(json!({ "msg": "ok" })))
}

pub async fn delete_comment_by_coid(
    State(state): State<Arc<AppState>>,
    PMEditor(_): PMEditor,
    Path(coid): Path<u32>,
) -> Result<Json<Value>, FieldError> {
    let comment = match db::get_comment_by_coid(&state, coid).await {
        Ok(Some(comment)) => comment,
        _ => return Err(FieldError::NotFound("coid".to_string())),
    };
    let cid = comment.cid;
    let _ = db::update_content_count_decrease_by_cid(&state, cid).await?;

    let _ = db::delete_comment_by_coid(&state, coid).await?;
    Ok(Json(json!({ "msg": "ok" })))
}
