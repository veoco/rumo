use std::sync::Arc;
use std::time::SystemTime;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Json;
use hmac::{Hmac, Mac};
use jwt::SignWithKey;
use serde_json::{json, Value};
use sha2::Sha256;

use super::db;
use super::forms::{
    OptionCreate, OptionModify, TokenData, UserLogin, UserModify, UserRegister, UsersQuery,
};
use super::utils::{authenticate_user, hash};
use crate::common::errors::{AuthError, FieldError};
use crate::common::extractors::{PMAdministrator, PMSubscriber, ValidatedJson, ValidatedQuery};
use crate::AppState;

pub async fn login_for_access_token(
    State(state): State<Arc<AppState>>,
    ValidatedJson(user_login): ValidatedJson<UserLogin>,
) -> Result<Json<Value>, AuthError> {
    if let Some(user) = authenticate_user(&state, &user_login).await {
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

        let _ = db::update_user_by_uid_for_activity(&state, user.uid, now as u32).await;

        return Ok(Json(
            json!({"access_token": access_token, "token_type": "Bearer"}),
        ));
    }
    Err(AuthError::WrongCredentials)
}

pub async fn register(
    State(state): State<Arc<AppState>>,
    ValidatedJson(user_register): ValidatedJson<UserRegister>,
) -> Result<(StatusCode, Json<Value>), FieldError> {
    let _ = db::create_user_with_user_register(&state, &user_register).await?;
    return Ok((StatusCode::CREATED, Json(json!({ "msg": "ok" }))));
}

pub async fn list_users(
    State(state): State<Arc<AppState>>,
    PMAdministrator(_): PMAdministrator,
    ValidatedQuery(q): ValidatedQuery<UsersQuery>,
) -> Result<Json<Value>, FieldError> {
    let all_count = db::get_users_count(&state).await;

    let page = q.page.unwrap_or(1);
    let page_size = q.page_size.unwrap_or(10);
    let order_by = q.order_by.unwrap_or("-uid".to_string());

    let (users, count) =
        db::get_users_by_list_query(&state, page_size as u64, page as u64, order_by).await?;
    Ok(Json(json!({
        "page": page,
        "page_size": page_size,
        "all_count": all_count,
        "count": count,
        "results": users
    })))
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
        if let Ok(Some(mut target_user)) = db::get_user_by_uid(&state, uid).await {
            target_user.password = None;
            Ok(Json(json!(target_user)))
        } else {
            Err(FieldError::InvalidParams("uid".to_string()))
        }
    } else {
        Err(FieldError::PermissionDeny)
    }
}

pub async fn modify_user_by_id(
    State(state): State<Arc<AppState>>,
    PMSubscriber(user): PMSubscriber,
    Path(uid): Path<u32>,
    ValidatedJson(user_modify): ValidatedJson<UserModify>,
) -> Result<Json<Value>, FieldError> {
    if (user.uid == uid && user.group == user_modify.group) || user.group == "administrator" {
        match user_modify.group.as_str() {
            "subscriber" | "contributor" | "editor" | "administrator" => {}
            _ => return Err(FieldError::InvalidParams("group".to_string())),
        }

        let exist_user = db::get_user_by_uid(&state, uid).await?;

        if exist_user.is_some() {
            if user_modify.password.is_none() {
                let user = db::update_user_by_uid_with_user_modify_for_data_without_password(
                    &state,
                    uid,
                    &user_modify,
                )
                .await?;
                Ok(Json(json!({
                    "msg": format!("{} infomation changed", user.uid)
                })))
            } else {
                let password = user_modify.password.unwrap();
                let hashed_password = hash(&password);

                let user =
                    db::update_user_by_uid_for_password(&state, uid, &hashed_password).await?;
                Ok(Json(json!({
                    "msg": format!("{} password changed", user.uid)
                })))
            }
        } else {
            Err(FieldError::InvalidParams("uid".to_string()))
        }
    } else {
        Err(FieldError::PermissionDeny)
    }
}

pub async fn delete_user_by_id(
    State(state): State<Arc<AppState>>,
    PMAdministrator(_): PMAdministrator,
    Path(uid): Path<u32>,
) -> Result<Json<Value>, FieldError> {
    let exist_user = db::get_user_by_uid(&state, uid).await?;
    if exist_user.is_none() {
        return Err(FieldError::InvalidParams("uid".to_string()));
    }

    let _ = db::delete_user_by_uid(&state, uid).await?;
    Ok(Json(json!({"msg": "ok"})))
}

pub async fn list_options(
    State(state): State<Arc<AppState>>,
    PMSubscriber(user): PMSubscriber,
    Path(uid): Path<u32>,
) -> Result<Json<Value>, FieldError> {
    if uid != user.uid && !(user.group == "administrator") {
        return Err(FieldError::PermissionDeny);
    }

    let options = db::get_options_by_uid(&state, uid).await?;
    Ok(Json(json!({
        "page": 1,
        "page_size": options.len(),
        "all_count": options.len(),
        "count": options.len(),
        "results": options
    })))
}

pub async fn get_option_by_uid_and_name(
    State(state): State<Arc<AppState>>,
    PMSubscriber(user): PMSubscriber,
    Path((uid, name)): Path<(u32, String)>,
) -> Result<Json<Value>, FieldError> {
    if user.uid != uid && !(user.group == "administrator") {
        return Err(FieldError::PermissionDeny);
    }

    let option = match db::get_option_by_uid_and_name(&state, uid, &name).await?{
        Some(option) => option,
        None => return Err(FieldError::NotFound("uid or name".to_string())),
    };
    Ok(Json(json!(option)))
}

pub async fn create_option_by_option_create(
    State(state): State<Arc<AppState>>,
    PMSubscriber(user): PMSubscriber,
    Path(_): Path<u32>,
    ValidatedJson(option_create): ValidatedJson<OptionCreate>,
) -> Result<(StatusCode, Json<Value>), FieldError> {
    match db::get_option_by_uid_and_name(&state, user.uid, &option_create.name).await {
        Ok(Some(_)) => return Err(FieldError::InvalidParams("name".to_string())),
        _ => (),
    };

    let _ = db::create_option_by_uid_with_option_create(&state, user.uid, &option_create).await?;
    Ok((StatusCode::CREATED, Json(json!({"msg": "ok"}))))
}

pub async fn modify_option_by_uid_and_name(
    State(state): State<Arc<AppState>>,
    PMSubscriber(user): PMSubscriber,
    Path((uid, name)): Path<(u32, String)>,
    ValidatedJson(option_modify): ValidatedJson<OptionModify>,
) -> Result<Json<Value>, FieldError> {
    if user.uid != uid && !(user.group == "administrator") {
        return Err(FieldError::PermissionDeny);
    }

    let option = db::get_option_by_uid_and_name(&state, uid, &name).await;
    if option.is_err() {
        return Err(FieldError::InvalidParams("uid or name".to_string()));
    }

    let _ =
        db::modify_option_by_uid_and_name_with_option_modify(&state, uid, &name, &option_modify)
            .await?;
    Ok(Json(json!({"msg": "ok"})))
}

pub async fn delete_option_by_uid_and_name(
    State(state): State<Arc<AppState>>,
    PMSubscriber(user): PMSubscriber,
    Path((uid, name)): Path<(u32, String)>,
) -> Result<Json<Value>, FieldError> {
    if user.uid != uid && !(user.group == "administrator") {
        return Err(FieldError::PermissionDeny);
    }

    match db::get_option_by_uid_and_name(&state, uid, &name).await {
        Ok(Some(_)) => (),
        _ => return Err(FieldError::InvalidParams("uid or name".to_string())),
    };

    let _ = db::delete_option_by_uid_and_name(&state, uid, &name).await?;
    Ok(Json(json!({"msg": "ok"})))
}
