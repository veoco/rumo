use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde_json::{json, Value};

use super::db;
use super::forms::{PostCreate, PostQuery, PostsQuery};
use crate::common::db as common_db;
use crate::common::errors::FieldError;
use crate::common::extractors::{PMContributor, PMVisitor, ValidatedJson, ValidatedQuery};
use crate::common::forms::FieldCreate;
use crate::AppState;

pub async fn create_post(
    State(state): State<Arc<AppState>>,
    PMContributor(user): PMContributor,
    ValidatedJson(mut post_create): ValidatedJson<PostCreate>,
) -> Result<(StatusCode, Json<Value>), FieldError> {
    if let Ok(Some(_)) = common_db::get_content_by_slug(&state, &post_create.slug).await {
        return Err(FieldError::AlreadyExist("slug".to_owned()));
    }

    if user.group == "contributor" {
        post_create.status = String::from("waiting");
    }

    let _ = db::create_post_by_post_create_with_uid(&state, &post_create, user.uid).await?;
    Ok((StatusCode::CREATED, Json(json!({ "msg": "ok" }))))
}

pub async fn modify_post_by_slug(
    State(state): State<Arc<AppState>>,
    PMContributor(user): PMContributor,
    Path(slug): Path<String>,
    ValidatedJson(mut post_modify): ValidatedJson<PostCreate>,
) -> Result<Json<Value>, FieldError> {
    let exist_post = match common_db::get_content_by_slug(&state, &slug).await {
        Ok(Some(p)) => p,
        _ => return Err(FieldError::NotFound("slug".to_owned())),
    };

    if slug != post_modify.slug {
        if let Ok(Some(_)) = common_db::get_content_by_slug(&state, &post_modify.slug).await {
            return Err(FieldError::AlreadyExist("post slug".to_owned()));
        }
    }

    if user.group == "contributor" {
        post_modify.status = String::from("waiting");
    }

    let _ =
        db::modify_post_by_post_create_with_exist_post(&state, &post_modify, &exist_post).await?;
    Ok(Json(json!({ "msg": "ok" })))
}

pub async fn list_posts(
    State(state): State<Arc<AppState>>,
    PMVisitor(user): PMVisitor,
    ValidatedQuery(q): ValidatedQuery<PostsQuery>,
) -> Result<Json<Value>, FieldError> {
    let private =
        q.private.unwrap_or(false) && (user.group == "editor" || user.group == "administrator");
    let own = q.own.unwrap_or(false) && user.group != "visitor";

    let all_count =
        common_db::get_contents_count_with_private(&state, private, own, &user, "post").await;

    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(10);
    let order_by = q.order_by.unwrap_or("-cid".to_string());

    let posts = db::get_contents_with_metas_user_and_fields_by_filter_and_list_query(
        &state, private, own, &user, page_size, page, &order_by, true,
    )
    .await?;
    Ok(Json(json!({
        "page": page,
        "page_size": page_size,
        "all_count": all_count,
        "count": posts.len(),
        "results": posts
    })))
}

pub async fn get_post_by_slug(
    State(state): State<Arc<AppState>>,
    PMVisitor(user): PMVisitor,
    Path(slug): Path<String>,
    ValidatedQuery(q): ValidatedQuery<PostQuery>,
) -> Result<Json<Value>, FieldError> {
    let admin = user.group == "editor" || user.group == "administrator";
    let private = q.private.unwrap_or(false) && admin;

    let post = db::get_content_with_metas_user_fields_by_slug_and_private(&state, &slug, private)
        .await
        .map_err(|_| FieldError::NotFound("slug".to_string()))?;

    let status = &post.status;
    if admin || status == "publish" || status == "hidden" || status == "password" {
        if post.password.is_none() {
            return Ok(Json(json!(post)));
        }

        let password = post.password.clone().unwrap();
        if let Some(query_password) = q.password {
            if password == query_password {
                return Ok(Json(json!(post)));
            }
        } else {
            return Err(FieldError::PasswordRequired);
        }
    } else {
        return Err(FieldError::PermissionDeny);
    }

    Err(FieldError::NotFound("slug".to_string()))
}

pub async fn delete_post_by_slug(
    State(state): State<Arc<AppState>>,
    PMContributor(user): PMContributor,
    Path(slug): Path<String>,
) -> Result<Json<Value>, FieldError> {
    let post = common_db::get_content_by_slug(&state, &slug).await?;

    if post.is_none() {
        return Err(FieldError::InvalidParams("slug".to_string()));
    }
    let post = post.unwrap();

    let admin = user.group == "editor" || user.group == "administrator";
    if post.author_id != user.uid && !admin {
        return Err(FieldError::PermissionDeny);
    }

    let _ = common_db::delete_fields_by_cid(&state, post.cid).await?;

    let _ = common_db::delete_content_by_cid(&state, post.cid).await?;
    Ok(Json(json!({ "msg": "ok" })))
}

pub async fn create_post_field_by_slug(
    State(state): State<Arc<AppState>>,
    PMContributor(user): PMContributor,
    Path(slug): Path<String>,
    ValidatedJson(field_create): ValidatedJson<FieldCreate>,
) -> Result<(StatusCode, Json<Value>), FieldError> {
    let exist_post = match common_db::get_content_by_slug(&state, &slug).await {
        Ok(Some(p)) => p,
        _ => return Err(FieldError::NotFound("slug".to_owned())),
    };

    let admin = user.group == "editor" || user.group == "administrator";
    if exist_post.author_id != user.uid && !admin {
        return Err(FieldError::PermissionDeny);
    }

    let _ = common_db::create_field_by_cid_with_field_create(&state, exist_post.cid, &field_create)
        .await?;
    Ok((StatusCode::CREATED, Json(json!({ "msg": "ok" }))))
}

pub async fn get_post_field_by_slug_and_name(
    State(state): State<Arc<AppState>>,
    Path((slug, name)): Path<(String, String)>,
) -> Result<Json<Value>, FieldError> {
    let exist_post = match common_db::get_content_by_slug(&state, &slug).await {
        Ok(Some(p)) => p,
        _ => return Err(FieldError::NotFound("slug".to_owned())),
    };

    let field = match common_db::get_field_by_cid_and_name(&state, exist_post.cid, &name).await {
        Ok(Some(f)) => f,
        _ => return Err(FieldError::NotFound("name".to_owned())),
    };
    Ok(Json(json!(field)))
}

pub async fn modify_post_field_by_slug_and_name(
    State(state): State<Arc<AppState>>,
    PMContributor(user): PMContributor,
    Path((slug, name)): Path<(String, String)>,
    ValidatedJson(field_modify): ValidatedJson<FieldCreate>,
) -> Result<Json<Value>, FieldError> {
    let exist_post = match common_db::get_content_by_slug(&state, &slug).await {
        Ok(Some(p)) => p,
        _ => return Err(FieldError::NotFound("slug".to_owned())),
    };

    let admin = user.group == "editor" || user.group == "administrator";
    if exist_post.author_id != user.uid && !admin {
        return Err(FieldError::PermissionDeny);
    }

    if name != field_modify.name {
        match common_db::get_field_by_cid_and_name(&state, exist_post.cid, &name).await {
            Ok(Some(f)) => f,
            _ => return Err(FieldError::NotFound("name".to_owned())),
        };
    }

    let _ = common_db::modify_field_by_cid_and_name_with_field_create(
        &state,
        exist_post.cid,
        &name,
        &field_modify,
    )
    .await?;
    Ok(Json(json!({ "msg": "ok" })))
}

pub async fn delete_post_field_by_slug_and_name(
    State(state): State<Arc<AppState>>,
    PMContributor(user): PMContributor,
    Path((slug, name)): Path<(String, String)>,
) -> Result<Json<Value>, FieldError> {
    let exist_post = match common_db::get_content_by_slug(&state, &slug).await {
        Ok(Some(p)) => p,
        _ => return Err(FieldError::NotFound("slug".to_owned())),
    };

    let admin = user.group == "editor" || user.group == "administrator";
    if exist_post.author_id != user.uid && !admin {
        return Err(FieldError::PermissionDeny);
    }

    match common_db::get_field_by_cid_and_name(&state, exist_post.cid, &name).await {
        Ok(Some(f)) => f,
        _ => return Err(FieldError::NotFound("name".to_owned())),
    };

    let _ = common_db::delete_field_by_cid_and_name(&state, exist_post.cid, &name).await?;
    Ok(Json(json!({ "msg": "ok" })))
}
