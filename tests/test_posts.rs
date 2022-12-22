use axum::http::StatusCode;
use serde_json::json;

mod common;
use common::{admin_post, get};

#[tokio::test]
async fn create_then_list_posts_success() {
    let (status_code, body) = get("/api/posts/?page=1&page_size=10&order_by=-cid").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let count = body.get("count").unwrap().as_u64().unwrap();

    let data = json!({
        "title": "testPost",
        "slug": "test-post",
        "created": 1666666666,
        "text": "testText",
        "status": "publish",
        "allowComment": "1",
        "allowPing": "1",
        "allowFeed": "1",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/posts/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, body) = get("/api/posts/?page=1&page_size=10&order_by=-cid").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let new_count = body.get("count").unwrap().as_u64().unwrap();
    assert!(new_count > count);
}

#[tokio::test]
async fn create_then_get_post_by_slug_success() {
    let data = json!({
        "title": "testPostCreate",
        "slug": "test-post-create",
        "created": 1666666666,
        "text": "testText",
        "status": "publish",
        "allowComment": "1",
        "allowPing": "1",
        "allowFeed": "1",
    })
    .to_string();

    let (status_code, _) = admin_post("/api/posts/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, _) = get("/api/posts/test-post-create").await;
    assert_eq!(status_code, StatusCode::OK);
}
