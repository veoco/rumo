use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde_json::{json, Value};
use std::sync::Arc;

use super::db;
use super::models::{PostCreate, PostQuery, PostsQuery};
use crate::common::errors::FieldError;
use crate::common::extractors::{PMContributor, PMVisitor, ValidatedJson, ValidatedQuery};
use crate::pages::db as page_db;
use crate::pages::models::FieldCreate;
use crate::AppState;

pub async fn create_post(
    State(state): State<Arc<AppState>>,
    PMContributor(user): PMContributor,
    ValidatedJson(mut post_create): ValidatedJson<PostCreate>,
) -> Result<(StatusCode, Json<Value>), FieldError> {
    let exist_post = db::get_post_by_slug(&state, &post_create.slug).await;
    if exist_post.is_some() {
        return Err(FieldError::AlreadyExist("slug".to_owned()));
    }

    if user.group == "contributor" {
        post_create.status = String::from("waiting");
    }

    let row_id = db::create_post_by_post_create_with_uid(&state, &post_create, user.uid).await?;
    Ok((StatusCode::CREATED, Json(json!({ "id": row_id }))))
}

pub async fn modify_page_by_slug(
    State(state): State<Arc<AppState>>,
    PMContributor(user): PMContributor,
    Path(slug): Path<String>,
    ValidatedJson(mut post_modify): ValidatedJson<PostCreate>,
) -> Result<Json<Value>, FieldError> {
    let exist_post = db::get_post_by_slug(&state, &slug).await;
    if exist_post.is_none() {
        return Err(FieldError::NotFound("slug".to_owned()));
    }
    let exist_post = exist_post.unwrap();

    let target_post = db::get_post_by_slug(&state, &post_modify.slug).await;
    if target_post.is_some() {
        return Err(FieldError::AlreadyExist("post slug".to_owned()));
    }

    if user.group == "contributor" {
        post_modify.status = String::from("waiting");
    }

    let row_id =
        db::modify_post_by_post_create_with_exist_post(&state, &post_modify, &exist_post).await?;
    Ok(Json(json!({ "id": row_id })))
}

pub async fn list_posts(
    State(state): State<Arc<AppState>>,
    PMVisitor(user): PMVisitor,
    ValidatedQuery(q): ValidatedQuery<PostsQuery>,
) -> Result<Json<Value>, FieldError> {
    let private =
        q.private.unwrap_or(false) && (user.group == "editor" || user.group == "administrator");
    let own = q.own.unwrap_or(false) && user.group != "visitor";

    let filter_sql = if private && !own {
        String::from("")
    } else if !private && own {
        format!(
            r#" AND {contents_table}."authorId" == {}"#,
            user.uid,
            contents_table = &state.contents_table,
        )
    } else {
        format!(
            r#" AND {contents_table}."status" == 'publish' AND {contents_table}."password" IS NULL"#,
            contents_table = &state.contents_table,
        )
    };

    let all_count = db::get_posts_count_by_filter(&state, &filter_sql).await;

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

    let posts =
        db::get_posts_by_filter_and_list_query(&state, &filter_sql, page_size, offset, order_by)
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
    let private =
        q.private.unwrap_or(false) && (user.group == "editor" || user.group == "administrator");
    let private_sql = if private {
        String::from("")
    } else {
        format!(
            r#" AND ({contents_table}."status" == 'publish' OR {contents_table}."status" == 'password' OR {contents_table}."status" == 'hidden')"#,
            contents_table = &state.contents_table,
        )
    };

    let post = db::get_post_by_slug_and_private(&state, &slug, &private_sql)
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
    let post = db::get_post_by_slug(&state, &slug).await;

    if post.is_none() {
        return Err(FieldError::InvalidParams("slug".to_string()));
    }
    let post = post.unwrap();

    let admin = user.group == "editor" || user.group == "administrator";
    if post.authorId != user.uid && !admin {
        return Err(FieldError::PermissionDeny);
    }

    let _ = page_db::delete_fields_by_cid(&state, post.cid).await?;

    let row_id = page_db::delete_content_by_cid(&state, post.cid).await?;
    Ok(Json(json!({ "id": row_id })))
}

pub async fn create_post_field_by_slug(
    State(state): State<Arc<AppState>>,
    PMContributor(user): PMContributor,
    Path(slug): Path<String>,
    ValidatedJson(field_create): ValidatedJson<FieldCreate>,
) -> Result<(StatusCode, Json<Value>), FieldError> {
    let exist_post = db::get_post_by_slug(&state, &slug).await;
    if exist_post.is_none() {
        return Err(FieldError::AlreadyExist("slug".to_owned()));
    }
    let exist_post = exist_post.unwrap();

    let admin = user.group == "editor" || user.group == "administrator";
    if exist_post.authorId != user.uid && !admin {
        return Err(FieldError::PermissionDeny);
    }

    let row_id =
        page_db::create_field_by_cid_with_field_create(&state, exist_post.cid, &field_create)
            .await?;
    Ok((StatusCode::CREATED, Json(json!({ "id": row_id }))))
}

pub async fn get_post_field_by_slug_and_name(
    State(state): State<Arc<AppState>>,
    Path((slug, name)): Path<(String, String)>,
) -> Result<Json<Value>, FieldError> {
    let exist_post = db::get_post_by_slug(&state, &slug).await;
    if exist_post.is_none() {
        return Err(FieldError::NotFound("slug".to_owned()));
    }
    let exist_post = exist_post.unwrap();

    let field = page_db::get_field_by_cid_and_name(&state, exist_post.cid, &name).await;
    if field.is_none() {
        return Err(FieldError::NotFound("name".to_owned()));
    }
    Ok(Json(json!(field)))
}

pub async fn modify_post_field_by_slug_and_name(
    State(state): State<Arc<AppState>>,
    PMContributor(user): PMContributor,
    Path((slug, name)): Path<(String, String)>,
    ValidatedJson(field_create): ValidatedJson<FieldCreate>,
) -> Result<Json<Value>, FieldError> {
    let exist_post = db::get_post_by_slug(&state, &slug).await;
    if exist_post.is_none() {
        return Err(FieldError::InvalidParams("slug".to_owned()));
    }
    let exist_post = exist_post.unwrap();

    let admin = user.group == "editor" || user.group == "administrator";
    if exist_post.authorId != user.uid && !admin {
        return Err(FieldError::PermissionDeny);
    }

    let exist_field = page_db::get_field_by_cid_and_name(&state, exist_post.cid, &name).await;
    if exist_field.is_none() {
        return Err(FieldError::InvalidParams("name".to_owned()));
    }

    let row_id = page_db::modify_field_by_cid_and_name_with_field_create(
        &state,
        exist_post.cid,
        &name,
        &field_create,
    )
    .await?;
    Ok(Json(json!({ "id": row_id })))
}

pub async fn delete_post_field_by_slug_and_name(
    State(state): State<Arc<AppState>>,
    PMContributor(user): PMContributor,
    Path((slug, name)): Path<(String, String)>,
) -> Result<Json<Value>, FieldError> {
    let exist_post = db::get_post_by_slug(&state, &slug).await;
    if exist_post.is_none() {
        return Err(FieldError::InvalidParams("slug".to_owned()));
    }
    let exist_post = exist_post.unwrap();

    let admin = user.group == "editor" || user.group == "administrator";
    if exist_post.authorId != user.uid && !admin {
        return Err(FieldError::PermissionDeny);
    }

    let field = page_db::get_field_by_cid_and_name(&state, exist_post.cid, &name).await;
    if field.is_none() {
        return Err(FieldError::InvalidParams("name".to_owned()));
    }

    let row_id = page_db::delete_field_by_cid_and_name(&state, exist_post.cid, &name).await?;
    Ok(Json(json!({ "id": row_id })))
}
