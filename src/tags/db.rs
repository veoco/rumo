use super::forms::TagCreate;
use crate::common::db as common_db;
use crate::common::errors::FieldError;
use crate::AppState;

pub async fn create_tag_by_tag_create(
    state: &AppState,
    tag_create: &TagCreate,
) -> Result<u64, FieldError> {
    let tag_parent = match tag_create.parent {
        Some(p) => p,
        _ => 0,
    };

    let insert_sql = format!(
        r#"
        INSERT INTO {metas_table} ("type", "name", "slug", "description", "parent")
        VALUES ('tag', ?1, ?2, ?3, ?4)
        "#,
        metas_table = &state.metas_table,
    );
    match sqlx::query(&insert_sql)
        .bind(&tag_create.name)
        .bind(&tag_create.slug)
        .bind(&tag_create.description)
        .bind(tag_parent)
        .execute(&state.pool)
        .await
    {
        Ok(r) => Ok(r.rows_affected()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn modify_tag_by_mid_and_tag_modify(
    state: &AppState,
    mid: i32,
    tag_modify: &TagCreate,
) -> Result<u64, FieldError> {
    let tag_parent = match tag_modify.parent {
        Some(mid) => match common_db::get_meta_by_mid(&state, mid).await {
            Some(_) => mid,
            None => return Err(FieldError::InvalidParams("parent".to_string())),
        },
        _ => 0,
    };

    let update_sql = format!(
        r#"
        UPDATE {metas_table}
        SET "name" = ?1, "slug" = ?2, "description" = ?3, "parent" = ?4
        WHERE {metas_table}."mid" == ?5
        "#,
        metas_table = &state.metas_table
    );
    match sqlx::query(&update_sql)
        .bind(&tag_modify.name)
        .bind(&tag_modify.slug)
        .bind(&tag_modify.description)
        .bind(&tag_parent)
        .bind(mid)
        .execute(&state.pool)
        .await
    {
        Ok(r) => Ok(r.rows_affected()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}
