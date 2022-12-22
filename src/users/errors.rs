use axum::{
    extract::rejection::{JsonRejection, QueryRejection},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidateRequestError {
    #[error(transparent)]
    ValidationError(#[from] validator::ValidationErrors),
    #[error(transparent)]
    AxumFormRejection(#[from] JsonRejection),
    #[error(transparent)]
    AxumQueryRejection(#[from] QueryRejection),
}

impl IntoResponse for ValidateRequestError {
    fn into_response(self) -> Response {
        match self {
            ValidateRequestError::ValidationError(_) => {
                let message = format!("Input validation error: {}", self).replace('\n', ", ");
                let message = Json(json!({ "msg": message }));
                (StatusCode::BAD_REQUEST, message)
            }
            ValidateRequestError::AxumFormRejection(_) => (
                StatusCode::BAD_REQUEST,
                Json(json!({"msg": "Invalid json"})),
            ),
            ValidateRequestError::AxumQueryRejection(_) => (
                StatusCode::BAD_REQUEST,
                Json(json!({"msg": "Invalid query params"})),
            ),
        }
        .into_response()
    }
}

#[derive(Debug)]
pub enum AuthError {
    WrongCredentials,
    InvalidToken,
    PermissionDeny,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        match self {
            AuthError::WrongCredentials => (
                StatusCode::UNAUTHORIZED,
                Json(json!({"msg": "Wrong credentials"})),
            ),
            AuthError::InvalidToken => (
                StatusCode::BAD_REQUEST,
                Json(json!({"msg": "Invalid token"})),
            ),
            AuthError::PermissionDeny => (
                StatusCode::FORBIDDEN,
                Json(json!({"msg": "Permission deny"})),
            ),
        }
        .into_response()
    }
}

#[derive(Debug)]
pub enum FieldError {
    AlreadyExist(String),
    PermissionDeny,
    InvalidParams(String),
    DatabaseFailed(String),
}

impl IntoResponse for FieldError {
    fn into_response(self) -> Response {
        match self {
            FieldError::AlreadyExist(field) => (
                StatusCode::BAD_REQUEST,
                Json(json!({ "msg": format!("{} already exist", field) })),
            ),
            FieldError::PermissionDeny => (
                StatusCode::FORBIDDEN,
                Json(json!({"msg": "Permission deny"})),
            ),
            FieldError::InvalidParams(field) => (
                StatusCode::BAD_REQUEST,
                Json(json!({ "msg": format!("Invalid {}", field) })),
            ),
            FieldError::DatabaseFailed(s) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "msg": format!("{}", s) })),
            )
        }
        .into_response()
    }
}
