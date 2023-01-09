use axum::Router;
use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use hyper::body::to_bytes;
use serde_json::{json, Value};
use sqlx::AnyPool;
use std::env;
use tower::ServiceExt;

use rumo::{app, AppState};

async fn setup_state() -> AppState {
    let pool = AnyPool::connect(&env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let secret_key = env::var("SECRET_KEY").unwrap();
    let access_token_expire_secondes = 3600 * 24 * 30;
    let preload_index = false;
    let index_page = "".to_string();
    let upload_root = ".".to_string();
    let read_only = false;

    let table_prefix = env::var("TABLE_PREFIX").unwrap_or("typecho_".to_string());
    let comments_table = format!("{}comments", table_prefix);
    let contents_table = format!("{}contents", table_prefix);
    let fields_table = format!("{}fields", table_prefix);
    let metas_table = format!("{}metas", table_prefix);
    let options_table = format!("{}options", table_prefix);
    let relationships_table = format!("{}relationships", table_prefix);
    let users_table = format!("{}users", table_prefix);

    AppState {
        pool,
        secret_key,
        access_token_expire_secondes,
        upload_root,
        read_only,
        preload_index,
        index_page,
        
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

#[allow(dead_code)]
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
pub async fn admin_delete(url: &str) -> (StatusCode, Option<Value>) {
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
        .method(http::Method::DELETE)
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

#[allow(dead_code)]
pub async fn admin_post_file(url: &str, data: Vec<u8>) -> (StatusCode, Option<Value>) {
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
        .header(http::header::CONTENT_TYPE, "multipart/form-data; boundary=testfileboundary")
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
pub fn get_multipart(filename: &str, content_type: &str) -> Vec<u8> {
    let boundary = "testfileboundary";
    let data = format!(
        "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\nContent-Type: {content_type}\r\n\r\naabbccddeeff\r\n--{boundary}--\r\n"
    );
    let data = data.into_bytes();

    data
}
