use axum::http::StatusCode;
use serde_json::json;

mod common;
use common::{admin_post, admin_patch, get};

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
    })
    .to_string();

    let (status_code, _) = admin_post("/api/posts/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, _) = get("/api/posts/test-post-create").await;
    assert_eq!(status_code, StatusCode::OK);
}

#[tokio::test]
async fn create_then_modify_post_by_slug_success() {
    let data = json!({
        "title": "testPostModify",
        "slug": "test-post-modify",
        "created": 1666666666,
        "text": "testText",
        "status": "publish",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/posts/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, _) = get("/api/posts/test-post-modify").await;
    assert_eq!(status_code, StatusCode::OK);

    let data = json!({
        "title": "testPostModied",
        "slug": "test-post-modified",
        "created": 1666666666,
        "text": "testTextModified",
        "status": "publish",
    })
    .to_string();
    let (status_code, _) = admin_patch("/api/posts/test-post-modify", data).await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, _) = get("/api/posts/test-post-modified").await;
    assert_eq!(status_code, StatusCode::OK);
}
