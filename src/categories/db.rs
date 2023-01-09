use sqlx::any::AnyKind;

use super::forms::CategoryCreate;
use crate::common::db as common_db;
use crate::common::errors::FieldError;
use crate::AppState;

pub async fn create_category_by_category_create(
    state: &AppState,
    category_create: &CategoryCreate,
) -> Result<u64, FieldError> {
    let category_parent = match category_create.parent {
        Some(mid) => match common_db::get_meta_by_mid(&state, mid).await {
            Some(_) => mid,
            None => return Err(FieldError::InvalidParams("parent".to_string())),
        },
        _ => 0,
    };

    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            INSERT INTO {metas_table} ("type", "name", "slug", "description", "parent")
            VALUES ('category', $1, $2, $3, $4)
            "#,
            metas_table = &state.metas_table
        ),
        _ => format!(
            r#"
            INSERT INTO {metas_table} ("type", "name", "slug", "description", "parent")
            VALUES ('category', ?, ?, ?, ?)
            "#,
            metas_table = &state.metas_table
        ),
    };
    match sqlx::query(&sql)
        .bind(&category_create.name)
        .bind(&category_create.slug)
        .bind(&category_create.description)
        .bind(&category_parent)
        .execute(&state.pool)
        .await
    {
        Ok(r) => Ok(r.rows_affected()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn modify_category_by_mid_and_category_modify(
    state: &AppState,
    mid: i32,
    category_modify: &CategoryCreate,
) -> Result<u64, FieldError> {
    let category_parent = match category_modify.parent {
        Some(mid) => match common_db::get_meta_by_mid(&state, mid).await {
            Some(_) => mid,
            None => return Err(FieldError::InvalidParams("parent".to_string())),
        },
        _ => 0,
    };

    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            UPDATE {metas_table}
            SET "name" = $1, "slug" = $2, "description" = $3, "parent" = $4
            WHERE "mid" == ?5
            "#,
            metas_table = &state.metas_table
        ),
        _ => format!(
            r#"
            UPDATE {metas_table}
            SET "name" = ?, "slug" = ?, "description" = ?, "parent" = ?
            WHERE "mid" == ?
            "#,
            metas_table = &state.metas_table
        ),
    };
    match sqlx::query(&sql)
        .bind(&category_modify.name)
        .bind(&category_modify.slug)
        .bind(&category_modify.description)
        .bind(&category_parent)
        .bind(mid)
        .execute(&state.pool)
        .await
    {
        Ok(r) => Ok(r.rows_affected()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}
