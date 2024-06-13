use std::env;

use axum::Router;
use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use http_body_util::BodyExt;
use minijinja::Environment;
use sea_orm::Database;
use serde_json::{json, Value};
use tower::ServiceExt;

use rumo::{app, AppState, INDEX_TPL};

async fn setup_state() -> AppState {
    let conn = Database::connect(&env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let secret_key = env::var("SECRET_KEY").unwrap();
    let access_token_expire_secondes = 3600 * 24 * 30;
    let preload_index = false;
    let mut jinja_env = Environment::new();
    jinja_env.add_template("index.html", &INDEX_TPL).unwrap();
    let upload_root = ".".to_string();
    let read_only = false;

    AppState {
        conn,
        secret_key,
        access_token_expire_secondes,
        upload_root,
        read_only,
        preload_index,
        jinja_env,
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
    let body = response.into_body().collect().await.unwrap().to_bytes();
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
    let body = response.into_body().collect().await.unwrap().to_bytes();
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
    let body = response.into_body().collect().await.unwrap().to_bytes();
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
    let body = response.into_body().collect().await.unwrap().to_bytes();
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
    let body = response.into_body().collect().await.unwrap().to_bytes();
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
    let body = response.into_body().collect().await.unwrap().to_bytes();
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
    let body = response.into_body().collect().await.unwrap().to_bytes();
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
    let body = response.into_body().collect().await.unwrap().to_bytes();
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
    let body = response.into_body().collect().await.unwrap().to_bytes();
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
    let body = response.into_body().collect().await.unwrap().to_bytes();
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
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();
    let token = body.get("access_token").unwrap().as_str().unwrap();

    let app = setup_app(state.clone()).await;

    let request = Request::builder()
        .method(http::Method::POST)
        .uri(url)
        .header(
            http::header::CONTENT_TYPE,
            "multipart/form-data; boundary=testfileboundary",
        )
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::from(data))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    let status_code = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body = serde_json::from_slice(&body).unwrap_or(None);
    (status_code, body)
}

#[allow(dead_code)]
pub async fn admin_patch_file(url: &str, data: Vec<u8>) -> (StatusCode, Option<Value>) {
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
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();
    let token = body.get("access_token").unwrap().as_str().unwrap();

    let app = setup_app(state.clone()).await;

    let request = Request::builder()
        .method(http::Method::PATCH)
        .uri(url)
        .header(
            http::header::CONTENT_TYPE,
            "multipart/form-data; boundary=testfileboundary",
        )
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::from(data))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    let status_code = response.status();
    let body = response.into_body().collect().await.unwrap().to_bytes();
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
