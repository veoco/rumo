use sea_orm::*;

use super::forms::TagCreate;
use crate::common::db as common_db;
use crate::common::errors::FieldError;
use crate::entity::meta;
use crate::AppState;

pub async fn create_tag_by_tag_create(
    state: &AppState,
    tag_create: &TagCreate,
) -> Result<meta::ActiveModel, FieldError> {
    let tag_parent = match tag_create.parent {
        Some(p) => p,
        _ => 0,
    };

    meta::ActiveModel {
        r#type: Set("tag".to_string()),
        name: Set(Some(tag_create.name.clone())),
        slug: Set(Some(tag_create.slug.clone())),
        description: Set(tag_create.description.clone()),
        parent: Set(tag_parent),
        ..Default::default()
    }
    .save(&state.conn)
    .await
    .map_err(|_| FieldError::DatabaseFailed("create tag failed".to_string()))
}

pub async fn modify_tag_by_mid_and_tag_modify(
    state: &AppState,
    mid: u32,
    tag_modify: &TagCreate,
) -> Result<meta::Model, FieldError> {
    let tag_parent = match tag_modify.parent {
        Some(mid) => match common_db::get_meta_by_mid(&state, mid).await {
            Ok(Some(_)) => mid,
            _ => return Err(FieldError::InvalidParams("parent".to_string())),
        },
        _ => 0,
    };

    let exist_tag = match common_db::get_meta_by_mid(&state, mid).await {
        Ok(Some(tag)) => tag,
        _ => return Err(FieldError::InvalidParams("mid".to_string())),
    };

    let mut t = meta::ActiveModel::from(exist_tag);
    t.name = Set(Some(tag_modify.name.clone()));
    t.slug = Set(Some(tag_modify.slug.clone()));
    t.description = Set(tag_modify.description.clone());
    t.parent = Set(tag_parent);
    t.update(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("modify tag failed".to_string()))
}
