use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use serde_json::json;
use tower::ServiceExt;

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
