use axum::{
    body::Body,
    http::{self, Request, StatusCode},
};
use hyper::body::to_bytes;
use serde_json::{json, Value};
use tower::ServiceExt;

mod common;
use common::{setup_app, setup_state};

#[tokio::test]
async fn create_then_list_posts_success() {
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
        .uri("/api/posts/?page=1&page_size=10&order_by=-cid")
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body()).await.unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    let count = body.get("count").unwrap().as_u64().unwrap();

    let app = setup_app(state.clone()).await;
    let data = json!({
        "title": "testPost",
        "slug": "test-post",
        "created": 1666666666,
        "text": "testText",
        "status": "publish",
        "allowComment": "1",
        "allowPing": "1",
        "allowFeed": "1",
    }).to_string();
    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/api/posts/")
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::from(data))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let app = setup_app(state.clone()).await;
    let request = Request::builder()
        .method(http::Method::GET)
        .uri("/api/posts/?page=1&page_size=10&order_by=-cid")
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let body = to_bytes(response.into_body()).await.unwrap();
    let body: Value = serde_json::from_slice(&body).unwrap();
    let new_count = body.get("count").unwrap().as_u64().unwrap();
    assert!(new_count > count);
}

#[tokio::test]
async fn create_then_get_post_by_slug_success() {
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
    let data = json!({
        "title": "testPostCreate",
        "slug": "test-post-create",
        "created": 1666666666,
        "text": "testText",
        "status": "publish",
        "allowComment": "1",
        "allowPing": "1",
        "allowFeed": "1",
    }).to_string();
    let request = Request::builder()
        .method(http::Method::POST)
        .uri("/api/posts/")
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::from(data))
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);

    let app = setup_app(state.clone()).await;
    let request = Request::builder()
        .method(http::Method::GET)
        .uri("/api/posts/test-post-create")
        .header(http::header::CONTENT_TYPE, "application/json")
        .header(http::header::AUTHORIZATION, format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let response = app.oneshot(request).await.unwrap();
    assert_eq!(response.status(), StatusCode::OK);
}
