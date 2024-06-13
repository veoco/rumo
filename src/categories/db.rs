use sea_orm::*;

use super::forms::CategoryCreate;
use crate::common::db as common_db;
use crate::common::errors::FieldError;
use crate::entity::meta;
use crate::AppState;

pub async fn create_category_by_category_create(
    state: &AppState,
    category_create: &CategoryCreate,
) -> Result<meta::ActiveModel, FieldError> {
    let category_parent = match category_create.parent {
        Some(mid) => match common_db::get_meta_by_mid(&state, mid).await {
            Ok(Some(_)) => mid,
            _ => return Err(FieldError::InvalidParams("parent".to_string())),
        },
        _ => 0,
    };

    meta::ActiveModel {
        r#type: Set("category".to_string()),
        name: Set(Some(category_create.name.clone())),
        slug: Set(Some(category_create.slug.clone())),
        description: Set(category_create.description.clone()),
        parent: Set(category_parent),
        ..Default::default()
    }
    .save(&state.conn)
    .await
    .map_err(|_| FieldError::DatabaseFailed("create category failed".to_string()))
}

pub async fn modify_category_by_mid_and_category_modify(
    state: &AppState,
    mid: u32,
    category_modify: &CategoryCreate,
) -> Result<meta::Model, FieldError> {
    let category_parent = match category_modify.parent {
        Some(mid) => match common_db::get_meta_by_mid(&state, mid).await {
            Ok(Some(_)) => mid,
            _ => return Err(FieldError::InvalidParams("parent".to_string())),
        },
        _ => 0,
    };

    let exist_category = match common_db::get_meta_by_mid(&state, mid).await {
        Ok(Some(category)) => category,
        _ => return Err(FieldError::InvalidParams("mid".to_string())),
    };

    let mut c = meta::ActiveModel::from(exist_category);
    c.name = Set(Some(category_modify.name.clone()));
    c.slug = Set(Some(category_modify.slug.clone()));
    c.description = Set(category_modify.description.clone());
    c.parent = Set(category_parent);
    c.update(&state.conn)
        .await
        .map_err(|_| FieldError::DatabaseFailed("modify category failed".to_string()))
}
