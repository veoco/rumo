use sqlx::any::AnyKind;
use std::time::SystemTime;

use super::models::{User, UserModify, UserRegister};
use super::utils::hash;
use crate::common::errors::FieldError;
use crate::AppState;

pub async fn get_user_by_mail(state: &AppState, mail: &str) -> Option<User> {
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            SELECT *
            FROM {users_table}
            WHERE "mail" = $1
            "#,
            users_table = &state.users_table
        ),
        AnyKind::MySql => format!(
            r#"
            SELECT *
            FROM {users_table}
            WHERE `mail` = ?
            "#,
            users_table = &state.users_table
        ),
        _ => format!(
            r#"
            SELECT *
            FROM {users_table}
            WHERE "mail" = ?
            "#,
            users_table = &state.users_table
        ),
    };
    if let Ok(user) = sqlx::query_as::<_, User>(&sql)
        .bind(mail)
        .fetch_one(&state.pool)
        .await
    {
        Some(user)
    } else {
        None
    }
}

pub async fn get_user_by_uid(state: &AppState, uid: i32) -> Option<User> {
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            SELECT *
            FROM {users_table}
            WHERE "uid" = $1
            "#,
            users_table = &state.users_table
        ),
        AnyKind::MySql => format!(
            r#"
            SELECT *
            FROM {users_table}
            WHERE `uid` = ?
            "#,
            users_table = &state.users_table
        ),
        _ => format!(
            r#"
            SELECT *
            FROM {users_table}
            WHERE "uid" = ?
            "#,
            users_table = &state.users_table
        ),
    };
    if let Ok(user) = sqlx::query_as::<_, User>(&sql)
        .bind(uid)
        .fetch_one(&state.pool)
        .await
    {
        Some(user)
    } else {
        None
    }
}

pub async fn delete_user_by_uid(state: &AppState, uid: i32) -> Result<u64, FieldError> {
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            DELETE FROM {users_table}
            WHERE "uid" = $1
            "#,
            users_table = &state.users_table
        ),
        AnyKind::MySql => format!(
            r#"
            DELETE FROM {users_table}
            WHERE `uid` = ?
            "#,
            users_table = &state.users_table
        ),
        _ => format!(
            r#"
            DELETE FROM {users_table}
            WHERE "uid" = ?
            "#,
            users_table = &state.users_table
        ),
    };
    match sqlx::query(&sql).bind(uid).execute(&state.pool).await {
        Ok(r) => Ok(r.rows_affected()),
        Err(e) => return Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn update_user_by_uid_for_activity(state: &AppState, uid: i32, now: i32) {
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            UPDATE {users_table}
            SET "activated" = $1, "logged" = $2
            WHERE "uid" = $3
            "#,
            users_table = &state.users_table
        ),
        AnyKind::MySql => format!(
            r#"
            UPDATE {users_table}
            SET `activated` = ?, `logged` = ?
            WHERE `uid` = ?
            "#,
            users_table = &state.users_table
        ),
        _ => format!(
            r#"
            UPDATE {users_table}
            SET "activated" = ?, "logged" = ?
            WHERE "uid" = ?
            "#,
            users_table = &state.users_table
        ),
    };
    let _ = sqlx::query(&sql)
        .bind(now)
        .bind(now)
        .bind(uid)
        .execute(&state.pool)
        .await;
}

pub async fn update_user_by_uid_with_user_modify_for_data_without_password(
    state: &AppState,
    uid: i32,
    user_modify: &UserModify,
) -> Result<u64, FieldError> {
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            UPDATE {users_table}
            SET "name" = $1, "mail" = $2, "url" = $3, "screenName" = $4, "group" = $5
            WHERE "uid" = $6
            "#,
            users_table = &state.users_table
        ),
        AnyKind::MySql => format!(
            r#"
            UPDATE {users_table}
            SET `name` = ?, `mail` = ?, `url` = ?, `screenName` = ?, `group` = ?
            WHERE `uid` = ?
            "#,
            users_table = &state.users_table
        ),
        _ => format!(
            r#"
            UPDATE {users_table}
            SET "name" = ?, "mail" = ?, "url" = ?, "screenName" = ?, "group" = ?
            WHERE "uid" = ?
            "#,
            users_table = &state.users_table
        ),
    };
    match sqlx::query(&sql)
        .bind(&user_modify.name)
        .bind(&user_modify.mail)
        .bind(&user_modify.url)
        .bind(&user_modify.screenName)
        .bind(&user_modify.group)
        .bind(uid)
        .execute(&state.pool)
        .await
    {
        Ok(r) => Ok(r.rows_affected()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn update_user_by_uid_for_password(
    state: &AppState,
    uid: i32,
    hashed_password: &str,
) -> Result<u64, FieldError> {
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            UPDATE {users_table}
            SET "password" = $1
            WHERE "uid" = $2
            "#,
            users_table = &state.users_table
        ),
        AnyKind::MySql => format!(
            r#"
            UPDATE {users_table}
            SET `password` = ?
            WHERE `uid` = ?
            "#,
            users_table = &state.users_table
        ),
        _ => format!(
            r#"
            UPDATE {users_table}
            SET "password" = ?
            WHERE "uid" = ?
            "#,
            users_table = &state.users_table
        ),
    };
    match sqlx::query(&sql)
        .bind(hashed_password)
        .bind(uid)
        .execute(&state.pool)
        .await
    {
        Ok(r) => Ok(r.rows_affected()),
        Err(e) => Err(FieldError::DatabaseFailed(e.to_string())),
    }
}

pub async fn create_user_with_user_register(
    state: &AppState,
    user_register: &UserRegister,
) -> Result<u64, FieldError> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i32;
    let hashed_password = hash(&user_register.password);

    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            INSERT INTO {users_table} ("name", "mail", "url", "screenName", "password", "created", "group")
            VALUES ($1, $2, $3, $4, $5, $6, 'subscriber')
            "#,
            users_table = &state.users_table
        ),
        AnyKind::MySql => format!(
            r#"
            INSERT INTO {users_table} (`name`, `mail`, `url`, `screenName`, `password`, `created`, `group`)
            VALUES (?, ?, ?, ?, ?, ?, 'subscriber')
            "#,
            users_table = &state.users_table
        ),
        _ => format!(
            r#"
            INSERT INTO {users_table} ("name", "mail", "url", "screenName", "password", "created", "group")
            VALUES (?, ?, ?, ?, ?, ?, 'subscriber')
            "#,
            users_table = &state.users_table
        ),
    };
    if let Ok(r) = sqlx::query(&sql)
        .bind(&user_register.name)
        .bind(&user_register.mail)
        .bind(&user_register.url)
        .bind(&user_register.name)
        .bind(hashed_password)
        .bind(now)
        .execute(&state.pool)
        .await
    {
        Ok(r.rows_affected())
    } else {
        Err(FieldError::AlreadyExist("name or mail".to_owned()))
    }
}

pub async fn get_users_count(state: &AppState) -> i32 {
    let sql = match state.pool.any_kind() {
        _ => format!(
            r#"
            SELECT COUNT(*)
            FROM {users_table};
            "#,
            users_table = &state.users_table
        ),
    };
    let all_count = sqlx::query_scalar::<_, i32>(&sql)
        .fetch_one(&state.pool)
        .await
        .unwrap_or(0);
    all_count
}

pub async fn get_users_by_list_query(
    state: &AppState,
    page_size: i32,
    offset: i32,
    order_by: &str,
) -> Result<Vec<User>, FieldError> {
    let sql = match state.pool.any_kind() {
        AnyKind::Postgres => format!(
            r#"
            SELECT *
            FROM {users_table}
            ORDER BY {}
            LIMIT $1 OFFSET $2"#,
            order_by,
            users_table = &state.users_table
        ),
        _ => format!(
            r#"
            SELECT *
            FROM {users_table}
            ORDER BY {}
            LIMIT ? OFFSET ?"#,
            order_by,
            users_table = &state.users_table
        ),
    };

    match sqlx::query_as::<_, User>(&sql)
        .bind(page_size)
        .bind(offset)
        .fetch_all(&state.pool)
        .await
    {
        Ok(users) => Ok(users),
        Err(e) => return Err(FieldError::DatabaseFailed(e.to_string())),
    }
}
