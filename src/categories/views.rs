use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use serde_json::{json, Value};
use std::sync::Arc;

use super::db;
use super::forms::{CategoryCreate, CategoryPostAdd};
use crate::common::db as common_db;
use crate::common::errors::FieldError;
use crate::common::extractors::{PMEditor, PMVisitor, ValidatedJson, ValidatedQuery};
use crate::common::forms::ListQuery;
use crate::posts::forms::PostsQuery;
use crate::AppState;

pub async fn create_category(
    State(state): State<Arc<AppState>>,
    PMEditor(_): PMEditor,
    ValidatedJson(category_create): ValidatedJson<CategoryCreate>,
) -> Result<(StatusCode, Json<Value>), FieldError> {
    let exist_cate = common_db::get_meta_by_slug(&state, &category_create.slug, false).await;
    if exist_cate.is_some() {
        return Err(FieldError::AlreadyExist("slug".to_owned()));
    }

    let _ = db::create_category_by_category_create(&state, &category_create).await?;
    Ok((StatusCode::CREATED, Json(json!({ "msg": "ok" }))))
}

pub async fn list_categories(
    State(state): State<Arc<AppState>>,
    ValidatedQuery(q): ValidatedQuery<ListQuery>,
) -> Result<Json<Value>, FieldError> {
    let all_count = common_db::get_metas_count(&state, false).await;

    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(10);
    let order_by = q.order_by.unwrap_or("-mid".to_string());

    let offset = (page - 1) * page_size;
    let order_by = match order_by.as_str() {
        "mid" => "mid",
        "-mid" => "mid DESC",
        "slug" => "slug",
        "-slug" => "slug DESC",
        f => return Err(FieldError::InvalidParams(f.to_string())),
    };

    let categories =
        common_db::get_metas_by_list_query(&state, page_size, offset, order_by, false).await?;
    Ok(Json(json!({
        "page": page,
        "page_size": page_size,
        "all_count": all_count,
        "count": categories.len(),
        "results": categories
    })))
}

pub async fn get_category_by_slug(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Result<Json<Value>, FieldError> {
    match common_db::get_meta_by_slug(&state, &slug, false).await {
        Some(category) => Ok(Json(json!(category))),
        None => Err(FieldError::NotFound("slug".to_string())),
    }
}

pub async fn modify_category_by_slug(
    State(state): State<Arc<AppState>>,
    PMEditor(_): PMEditor,
    Path(slug): Path<String>,
    ValidatedJson(category_modify): ValidatedJson<CategoryCreate>,
) -> Result<Json<Value>, FieldError> {
    let exist_cate = common_db::get_meta_by_slug(&state, &slug, false).await;
    if exist_cate.is_none() {
        return Err(FieldError::InvalidParams("slug".to_owned()));
    }
    let exist_cate = exist_cate.unwrap();

    let target_cate = common_db::get_meta_by_slug(&state, &category_modify.slug, false).await;
    if target_cate.is_some() {
        return Err(FieldError::InvalidParams("category slug".to_owned()));
    }

    let _ =
        db::modify_category_by_mid_and_category_modify(&state, exist_cate.mid, &category_modify)
            .await?;
    Ok(Json(json!({ "msg": "ok" })))
}

pub async fn delete_category_by_slug(
    State(state): State<Arc<AppState>>,
    PMEditor(_): PMEditor,
    Path(slug): Path<String>,
) -> Result<Json<Value>, FieldError> {
    let exist_cate = common_db::get_meta_by_slug(&state, &slug, false).await;
    if exist_cate.is_none() {
        return Err(FieldError::InvalidParams("slug".to_owned()));
    }
    let exist_cate = exist_cate.unwrap();

    let _ = common_db::delete_relationships_by_mid(&state, exist_cate.mid).await?;
    let _ = common_db::delete_meta_by_mid(&state, exist_cate.mid).await?;
    Ok(Json(json!({ "msg": "ok" })))
}

pub async fn add_post_to_category(
    State(state): State<Arc<AppState>>,
    PMEditor(_): PMEditor,
    Path(slug): Path<String>,
    ValidatedJson(category_post_add): ValidatedJson<CategoryPostAdd>,
) -> Result<(StatusCode, Json<Value>), FieldError> {
    let mid = match common_db::get_meta_by_slug(&state, &slug, false).await {
        Some(category) => category.mid,
        None => return Err(FieldError::InvalidParams("slug".to_string())),
    };

    let cid = match common_db::get_content_by_slug(&state, &category_post_add.slug).await {
        Some(post) => post.cid,
        None => return Err(FieldError::InvalidParams("post slug".to_string())),
    };

    let exist = common_db::check_relationship_by_cid_and_mid(&state, cid, mid).await?;

    if !exist {
        let _ = common_db::create_relationship_by_cid_and_mid(&state, cid, mid).await?;
        let _ = common_db::update_meta_by_mid_for_increase_count(&state, mid).await?;
        Ok((StatusCode::CREATED, Json(json!({"msg": "ok"}))))
    } else {
        Err(FieldError::AlreadyExist("slug".to_string()))
    }
}

pub async fn delete_post_from_category(
    State(state): State<Arc<AppState>>,
    PMEditor(_): PMEditor,
    Path((slug, post_slug)): Path<(String, String)>,
) -> Result<Json<Value>, FieldError> {
    let mid = match common_db::get_meta_by_slug(&state, &slug, false).await {
        Some(category) => category.mid,
        None => return Err(FieldError::InvalidParams("slug".to_string())),
    };

    let cid = match common_db::get_content_by_slug(&state, &post_slug).await {
        Some(post) => post.cid,
        None => return Err(FieldError::InvalidParams("post slug".to_string())),
    };

    let exist = common_db::check_relationship_by_cid_and_mid(&state, cid, mid).await?;

    if exist {
        let _ = common_db::delete_relationship_by_cid_and_mid(&state, cid, mid).await?;
        let _ = common_db::update_meta_by_mid_for_decrease_count(&state, mid).await?;
        Ok(Json(json!({"msg": "ok"})))
    } else {
        Err(FieldError::AlreadyExist("slug".to_string()))
    }
}

pub async fn list_category_posts_by_slug(
    State(state): State<Arc<AppState>>,
    PMVisitor(user): PMVisitor,
    Path(slug): Path<String>,
    ValidatedQuery(q): ValidatedQuery<PostsQuery>,
) -> Result<Json<Value>, FieldError> {
    let mid = match common_db::get_meta_by_slug(&state, &slug, false).await {
        Some(category) => category.mid,
        None => return Err(FieldError::InvalidParams("slug".to_string())),
    };

    let private =
        q.private.unwrap_or(false) && (user.group == "editor" || user.group == "administrator");
    let private_sql = if private {
        String::from("")
    } else {
        format!(r#" AND "status" = 'publish' AND "password" IS NULL"#)
    };

    let all_count =
        common_db::get_meta_posts_count_by_mid_with_private(&state, mid, &private_sql).await;

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

    let posts = common_db::get_contents_with_metas_user_and_fields_by_mid_list_query_and_private(
        &state,
        mid,
        &private_sql,
        page_size,
        offset,
        order_by,
        true,
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
