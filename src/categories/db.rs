use super::models::{Category, CategoryCreate};
use crate::posts::db as post_db;
use crate::posts::models::PostWithMeta;
use crate::users::errors::FieldError;
use crate::AppState;

pub async fn get_category_by_slug(state: &AppState, slug: &str) -> Option<Category> {
    let select_sql = format!(
        r#"
        SELECT *
        FROM {metas_table}
        WHERE {metas_table}."type" == 'category' AND {metas_table}."slug" == ?1
        "#,
        metas_table = &state.metas_table,
    );
    if let Ok(tag) = sqlx::query_as::<_, Category>(&select_sql)
        .bind(slug)
        .fetch_one(&state.pool)
        .await
    {
        Some(tag)
    } else {
        None
    }
}

pub async fn create_category_by_category_create(
    state: &AppState,
    category_create: &CategoryCreate,
) -> Result<i64, FieldError> {
    let category_parent = match category_create.parent {
        Some(p) => p,
        _ => 0,
    };

    let insert_sql = format!(
        r#"
        INSERT INTO {metas_table} ("type", "name", "slug", "description", "parent")
        VALUES ('category', ?1, ?2, ?3, ?4)
        "#,
        metas_table = &state.metas_table
    );
    match sqlx::query(&insert_sql)
        .bind(&category_create.name)
        .bind(&category_create.slug)
        .bind(&category_create.description)
        .bind(&category_parent)
        .execute(&state.pool)
        .await
    {
        Ok(r) => Ok(r.last_insert_rowid()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string()))
    }
}

pub async fn get_categories_count(state: &AppState) -> i32 {
    let all_sql = format!(
        r#"
        SELECT COUNT(*)
        FROM {metas_table}
        WHERE {metas_table}."type" == 'category'
        "#,
        metas_table = &state.metas_table,
    );
    let all_count = sqlx::query_scalar::<_, i32>(&all_sql)
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);
    all_count
}

pub async fn get_categories_by_list_query(
    state: &AppState,
    page_size: u32,
    offset: u32,
    order_by: &str,
) -> Result<Vec<Category>, FieldError> {
    let sql = format!(
        r#"
        SELECT *
        FROM {metas_table}
        WHERE {metas_table}."type" == 'category'
        ORDER BY {}
        LIMIT ?1 OFFSET ?2
        "#,
        order_by,
        metas_table = &state.metas_table,
    );
    match sqlx::query_as::<_, Category>(&sql)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
    {
        Ok(categories) => Ok(categories),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn update_meta_by_mid_for_increase_count(state: &AppState, mid: u32) -> Result<u64, FieldError> {
    let update_sql = format!(
        r#"
        UPDATE {metas_table}
        SET count=count+1
        WHERE {metas_table}."mid" == ?1
        "#,
        metas_table = &state.metas_table,
    );
    match sqlx::query(&update_sql)
        .bind(mid)
        .execute(&state.pool)
        .await
    {
        Ok(r) => Ok(r.rows_affected()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn update_meta_by_mid_for_decrease_count(state: &AppState, mid: u32) -> Result<u64, FieldError> {
    let update_sql = format!(
        r#"
        UPDATE {metas_table}
        SET count=count-1
        WHERE {metas_table}."mid" == ?1
        "#,
        metas_table = &state.metas_table,
    );
    match sqlx::query(&update_sql)
        .bind(mid)
        .execute(&state.pool)
        .await
    {
        Ok(r) => Ok(r.rows_affected()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn get_category_posts_count_by_mid(state: &AppState, mid: u32, private_sql: &str) -> i32 {
    let all_sql = format!(
        r#"
        SELECT COUNT(*)
        FROM {contents_table}
        JOIN {relationships_table} ON {contents_table}."cid" == {relationships_table}."cid"
        WHERE {contents_table}."type" == 'post' AND {relationships_table}."mid" == ?1{}
        "#,
        private_sql,
        contents_table = &state.contents_table,
        relationships_table = &state.relationships_table
    );
    let all_count = sqlx::query_scalar::<_, i32>(&all_sql)
        .bind(mid)
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);
    all_count
}

pub async fn get_category_posts_with_meta_by_mid_and_list_query(
    state: &AppState,
    mid: u32,
    private_sql: &str,
    page_size: u32,
    offset: u32,
    order_by: &str,
) -> Result<Vec<PostWithMeta>, FieldError> {
    let with_sql = post_db::get_with_sql(state);
    let sql = format!(
        r#"
        {with_sql}
        SELECT *
        FROM {contents_table}
        LEFT OUTER JOIN categories_json ON {contents_table}."cid" == categories_json."cid"
        LEFT OUTER JOIN tags_json ON {contents_table}."cid" == tags_json."cid"
        LEFT OUTER JOIN fields_json ON {contents_table}."cid" == fields_json."cid"
        LEFT OUTER JOIN {users_table} ON {contents_table}.authorId == {users_table}."uid"
        JOIN {relationships_table} ON {contents_table}.cid == {relationships_table}."cid"
        WHERE {contents_table}."type" == 'post' AND mid == ?1{}
        GROUP BY {contents_table}."cid"
        ORDER BY {contents_table}.{}
        LIMIT ?2 OFFSET ?3"#,
        private_sql,
        order_by,
        contents_table = &state.contents_table,
        relationships_table = &state.relationships_table,
        users_table = &state.users_table
    );
    match sqlx::query_as::<_, PostWithMeta>(&sql)
        .bind(mid)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
    {
        Ok(posts) => Ok(posts),
        Err(e) => return Err(FieldError::DatabaseFailed(e.to_string())),
    }
}