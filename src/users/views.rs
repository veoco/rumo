use axum::extract::{Path, Query, State};
use axum::response::Json;
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use serde_json::{json, Value};
use sha2::Sha256;
use std::sync::Arc;
use std::time::SystemTime;

use super::errors::{AuthError, FieldError};
use super::extractors::{PMAdministrator, PMSubscriber, ValidatedJson};
use super::models::{TokenData, User, UserLogin, UserRegister};
use super::utils::{authenticate_user, hash};
use crate::AppState;

pub async fn login_for_access_token(
    State(state): State<Arc<AppState>>,
    ValidatedJson(user_login): ValidatedJson<UserLogin>,
) -> Result<Json<Value>, AuthError> {
    if let Some(user) = authenticate_user(&state.pool, &user_login).await {
        let key: Hmac<Sha256> = Hmac::new_from_slice(state.secret_key.as_bytes()).unwrap();
        let now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let token_data = TokenData {
            sub: format!("{}", user.uid),
            exp: now + state.access_token_expire_secondes,
        };
        let access_token = token_data.sign_with_key(&key).unwrap();

        let _ = sqlx::query(
            r#"
            UPDATE typecho_users
            SET activated = ?1, logged = ?1
            WHERE uid = ?2
            "#
        ).bind(now as u32).bind(user.uid)
        .execute(&state.pool)
        .await;  

        return Ok(Json(
            json!({"access_token": access_token, "token_type": "Bearer"}),
        ));
    }
    Err(AuthError::WrongCredentials)
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    ValidatedJson(user_register): ValidatedJson<UserRegister>,
) -> Result<Json<Value>, FieldError> {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as u32;
    let hashed_password = hash(&user_register.password);

    if let Ok(r) = sqlx::query(
        r#"
        INSERT INTO typecho_users (name, mail, url, screenName, password, created, "group") VALUES (?1, ?2, ?3, ?1, ?4, ?5, ?6)"#,
    ).bind(user_register.name).bind(user_register.mail).bind(user_register.url).bind(hashed_password).bind(now).bind("subscriber")
    .execute(&state.pool).await {
        return Ok(Json(json!({"id": r.last_insert_rowid()})))
    }
    Err(FieldError::AlreadyExist("name or mail".to_owned()))
}

pub async fn list_users(
    State(state): State<Arc<AppState>>,
    PMAdministrator(_): PMAdministrator,
    Query(page): Query<u32>,
    Query(page_size): Query<u32>,
) -> Json<Value> {
    let all_count = sqlx::query_scalar::<_, i32>(
        r#"
        SELECT COUNT(*)
        FROM typecho_users;
        "#,
    )
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);

    if let Ok(users) = sqlx::query_as::<_, User>(
        r#"
        SELECT *
        FROM typecho_users
        ORDER BY uid
            "#,
    )
    .fetch_all(&state.pool)
    .await
    {
        return Json(json!({
            "page": page,
            "page_size": page_size,
            "all_count": all_count,
            "count": users.len(),
            "results": users
        }));
    }
    Json(json!({
        "page": page,
        "page_size": page_size,
        "all_count": all_count,
        "count": 0,
        "results": 0
    }))
}

pub async fn get_user_by_id(
    State(state): State<Arc<AppState>>,
    PMSubscriber(user): PMSubscriber,
    Path(uid): Path<u32>,
) -> Result<Json<Value>, FieldError> {
    if user.uid == uid {
        return Ok(Json(json!(user)));
    }

    if user.group == "administrator" {
        if let Ok(target_user) = sqlx::query_as::<_, User>(
            r#"
            SELECT *
            FROM typecho_users
            WHERE uid == ?1
                "#,
        )
        .bind(uid)
        .fetch_one(&state.pool)
        .await
        {
            return Ok(Json(json!(target_user)));
        }
        return Err(FieldError::InvalidParams("uid".to_string()));
    }
    Err(FieldError::PermissionDeny)
}
