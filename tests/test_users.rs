use axum::{
    body::{Body},
    http::{self, Request, StatusCode},
};
use serde_json::{json, Value};
use tower::ServiceExt;
use hyper::body::to_bytes;

mod common;
use common::{setup_app, setup_state};

#[tokio::test]
async fn index() {
    let state = setup_state().await;
    let app = setup_app(state).await;

    let response = app
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn login_failed() {
    let state = setup_state().await;
    let app = setup_app(state).await;

    let data = json!({"mail": "login_failed@test.local", "password": "password"}).to_string();
    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/api/users/token")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(data))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_success() {
    let state = setup_state().await;
    let app = setup_app(state.clone()).await;

    // register a normal user
    let data = json!({"name": "login_test","mail": "login_success@test.local", "url": "http://127.0.0.1", "password": "password"}).to_string();
    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/api/users")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(data))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let app = setup_app(state.clone()).await;
    let data = json!({"mail": "login_success@test.local", "password": "password"}).to_string();
    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/api/users/token")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(data))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn normal_user_change_success() {
    let state = setup_state().await;
    let app = setup_app(state.clone()).await;

    // register a normal user
    let data = json!({"name": "change_test","mail": "change_test@test.local", "url": "http://127.0.0.1", "password": "password"}).to_string();
    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/api/users")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(data))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body()).await.unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    let uid = body.get("id");
    assert!(uid.is_some());
    let uid = uid.unwrap().as_u64().unwrap();

    let app = setup_app(state.clone()).await;
    let data = json!({"mail": "change_test@test.local", "password": "password"}).to_string();
    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/api/users/token")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(data))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body()).await.unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    let token = body.get("access_token");
    assert!(token.is_some());

    let token = token.unwrap().as_str().unwrap();
    let app = setup_app(state.clone()).await;
    let data = json!({"name": "changed_test", "mail": "changed_test@test.local", "url": "http://127.0.0.1", "screenName": "changed_test", "group": "subscriber"}).to_string();
    let request = Request::builder()
        .method(http::Method::PATCH)
        .uri(format!("/api/users/{}", uid))
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::from(data))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let app = setup_app(state.clone()).await;
    let request = Request::builder()
        .method(http::Method::GET)
        .uri(format!("/api/users/{}", uid))
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body()).await.unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    let name = body.get("name");
    assert!(name.is_some());

    let name = name.unwrap().as_str().unwrap();
    assert_eq!(name, "changed_test");
}

#[tokio::test]
async fn list_users_success() {
    let state = setup_state().await;
    let app = setup_app(state.clone()).await;

    // login as admin
    let data = json!({"mail": "admin@local.host", "password": "admin"}).to_string();
    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/api/users/token")
        .header(http::header::CONTENT_TYPE, "application/json")
        .body(Body::from(data))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body()).await.unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    let token = body.get("access_token").unwrap().as_str().unwrap();

    let app = setup_app(state.clone()).await;
    let request = Request::builder()
        .method(http::Method::GET)
        .uri("/api/users/?page=1&page_size=10&order_by=-uid")
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
