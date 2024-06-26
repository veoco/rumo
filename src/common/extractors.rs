use axum::{
    async_trait,
    body::Body,
    extract::{rejection::JsonRejection, FromRef, FromRequest, FromRequestParts, Query},
    http::{request::Parts, Request},
    Json,
};
use serde::de::DeserializeOwned;
use std::sync::Arc;
use validator::Validate;

use super::errors::{AuthError, ValidateRequestError};
use crate::entity::user::Model as User;
use crate::users::utils::get_user;
use crate::AppState;

#[derive(Debug, Clone, Copy, Default)]
pub struct ValidatedJson<T>(pub T);

#[async_trait]
impl<S, T> FromRequest<S> for ValidatedJson<T>
where
    T: DeserializeOwned + Validate,
    S: Send + Sync,
    Json<T>: FromRequest<S, Rejection = JsonRejection>,
{
    type Rejection = ValidateRequestError;

    async fn from_request(req: Request<Body>, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state).await?;
        value.validate()?;
        Ok(ValidatedJson(value))
    }
}

pub struct ValidatedQuery<T>(pub T);

#[async_trait]
impl<T, S> FromRequestParts<Arc<S>> for ValidatedQuery<T>
where
    T: DeserializeOwned + Validate,
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = ValidateRequestError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<S>,
    ) -> Result<Self, Self::Rejection> {
        let Query(value) = Query::<T>::from_request_parts(parts, state).await?;
        value.validate()?;
        Ok(ValidatedQuery(value))
    }
}

pub struct PMVisitor(pub User);

#[async_trait]
impl<S> FromRequestParts<Arc<S>> for PMVisitor
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<S>,
    ) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);
        let visitor = User {
            uid: 0,
            name: None,
            password: None,
            mail: None,
            url: None,
            screen_name: None,
            created: 0,
            activated: 0,
            logged: 0,
            group: String::from("visitor"),
            auth_code: None,
        };
        let user = get_user(parts, state).await.unwrap_or(visitor);
        match user.group.as_str() {
            "visitor" | "subscriber" | "contributor" | "editor" | "administrator" => {
                return Ok(PMVisitor(user))
            }
            _ => return Err(AuthError::PermissionDeny),
        }
    }
}

pub struct PMSubscriber(pub User);

#[async_trait]
impl<S> FromRequestParts<Arc<S>> for PMSubscriber
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<S>,
    ) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);
        let user = get_user(parts, state).await?;
        match user.group.as_str() {
            "subscriber" | "contributor" | "editor" | "administrator" => {
                return Ok(PMSubscriber(user))
            }
            _ => return Err(AuthError::PermissionDeny),
        }
    }
}

pub struct PMContributor(pub User);

#[async_trait]
impl<S> FromRequestParts<Arc<S>> for PMContributor
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<S>,
    ) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);
        let user = get_user(parts, state).await?;
        match user.group.as_str() {
            "contributor" | "editor" | "administrator" => return Ok(PMContributor(user)),
            _ => return Err(AuthError::PermissionDeny),
        }
    }
}

pub struct PMEditor(pub User);

#[async_trait]
impl<S> FromRequestParts<Arc<S>> for PMEditor
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<S>,
    ) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);
        let user = get_user(parts, state).await?;
        match user.group.as_str() {
            "editor" | "administrator" => return Ok(PMEditor(user)),
            _ => return Err(AuthError::PermissionDeny),
        }
    }
}

pub struct PMAdministrator(pub User);

#[async_trait]
impl<S> FromRequestParts<Arc<S>> for PMAdministrator
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<S>,
    ) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);
        let user = get_user(parts, state).await?;
        match user.group.as_str() {
            "administrator" => return Ok(PMAdministrator(user)),
            _ => return Err(AuthError::PermissionDeny),
        }
    }
}
