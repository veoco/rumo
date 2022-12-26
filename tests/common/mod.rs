use axum::Router;
use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use hyper::body::to_bytes;
use serde_json::{json, Value};
use sqlx::sqlite::SqlitePool;
use std::env;
use tower::ServiceExt;

use rumo::{app, AppState};

async fn setup_state() -> AppState {
    let pool = SqlitePool::connect(&env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let secret_key = env::var("SECRET_KEY").unwrap();
    let access_token_expire_secondes = 3600 * 24 * 30;

    let table_prefix = env::var("TABLE_PREFIX").unwrap_or("typecho_".to_string());
    let comments_table = format!("{}_comments", table_prefix);
    let contents_table = format!("{}_contents", table_prefix);
    let fields_table = format!("{}_fields", table_prefix);
    let metas_table = format!("{}_metas", table_prefix);
    let options_table = format!("{}_options", table_prefix);
    let relationships_table = format!("{}_relationships", table_prefix);
    let users_table = format!("{}_users", table_prefix);

    AppState {
        pool,
        secret_key,
        access_token_expire_secondes,
        comments_table,
        contents_table,
        fields_table,
        metas_table,
        options_table,
        relationships_table,
        users_table,
    }
}

async fn setup_app(state: AppState) -> Router {
    app(Some(state)).await
}

pub async fn get(url: &str) -> (StatusCode, Option<Value>) {
    let state = setup_state().await;
    let app = setup_app(state.clone()).await;

    let request = Request::builder()
        .method(http::Method::GET)
        .uri(url)
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    let status_code = response.status();
    let body = to_bytes(response.into_body()).await.unwrap();
    let body = serde_json::from_slice(&body).unwrap_or(None);
    (status_code, body)
}

#[allow(dead_code)]
pub async fn post(url: &str, data: String) -> (StatusCode, Option<Value>) {
    let state = setup_state().await;
    let app = setup_app(state.clone()).await;

    let request = Request::builder()
        .method(http::Method::POST)
        .uri(url)
        .header(http::header::CONTENT_TYPE, "application/json")
        .header("User-Agent", "test")
        .header("X-Forwarded-For", "1.1.1.1, 2.2.2.2")
        .body(Body::from(data))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    let status_code = response.status();
    let body = to_bytes(response.into_body()).await.unwrap();
    let body = serde_json::from_slice(&body).unwrap_or(None);
    (status_code, body)
}

#[allow(dead_code)]
pub async fn admin_get(url: &str) -> (StatusCode, Option<Value>) {
    let state = setup_state().await;
    let app = setup_app(state.clone()).await;
    let login_data = json!({"mail": "admin@local.host", "password": "admin"}).to_string();
    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/api/users/token")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(login_data))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    let body = to_bytes(response.into_body()).await.unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    let token = body.get("access_token").unwrap().as_str().unwrap();

    let app = setup_app(state.clone()).await;

    let request = Request::builder()
        .method(http::Method::GET)
        .uri(url)
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    let status_code = response.status();
    let body = to_bytes(response.into_body()).await.unwrap();
    let body = serde_json::from_slice(&body).unwrap_or(None);
    (status_code, body)
}

#[allow(dead_code)]
pub async fn admin_post(url: &str, data: String) -> (StatusCode, Option<Value>) {
    let state = setup_state().await;
    let app = setup_app(state.clone()).await;
    let login_data = json!({"mail": "admin@local.host", "password": "admin"}).to_string();
    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/api/users/token")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(login_data))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    let body = to_bytes(response.into_body()).await.unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    let token = body.get("access_token").unwrap().as_str().unwrap();

    let app = setup_app(state.clone()).await;

    let request = Request::builder()
        .method(http::Method::POST)
        .uri(url)
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::from(data))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    let status_code = response.status();
    let body = to_bytes(response.into_body()).await.unwrap();
    let body = serde_json::from_slice(&body).unwrap_or(None);
    (status_code, body)
}

#[allow(dead_code)]
pub async fn admin_patch(url: &str, data: String) -> (StatusCode, Option<Value>) {
    let state = setup_state().await;
    let app = setup_app(state.clone()).await;
    let login_data = json!({"mail": "admin@local.host", "password": "admin"}).to_string();
    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/api/users/token")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(login_data))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    let body = to_bytes(response.into_body()).await.unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    let token = body.get("access_token").unwrap().as_str().unwrap();

    let app = setup_app(state.clone()).await;

    let request = Request::builder()
        .method(http::Method::PATCH)
        .uri(url)
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::from(data))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    let status_code = response.status();
    let body = to_bytes(response.into_body()).await.unwrap();
    let body = serde_json::from_slice(&body).unwrap_or(None);
    (status_code, body)
}
