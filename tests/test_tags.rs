use axum::http::StatusCode;
use serde_json::json;

mod common;
use common::{admin_post, get};

#[tokio::test]
async fn create_then_list_tags_success() {
    let (status_code, body) = get("/api/tags/?page=1&page_size=10&order_by=-mid").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let count = body.get("count").unwrap().as_u64().unwrap();

    let data = json!({"name": "testTag", "slug": "test-tag"}).to_string();
    let (status_code, _) = admin_post("/api/tags/", data).await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, body) = get("/api/tags/?page=1&page_size=10&order_by=-mid").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let new_count = body.get("count").unwrap().as_u64().unwrap();
    assert!(new_count > count);
}

#[tokio::test]
async fn create_then_get_tag_by_slug_success() {
    let data = json!({"name": "testTagCreate", "slug": "test-tag-create"}).to_string();
    let (status_code, _) = admin_post("/api/tags/", data).await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, _) = get("/api/tags/test-tag-create").await;
    assert_eq!(status_code, StatusCode::OK);
}

#[tokio::test]
async fn create_then_list_tag_posts_success() {
    let data = json!({"name": "testTagPost", "slug": "test-tag-post"}).to_string();
    let (status_code, _) = admin_post("/api/tags/", data).await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, body) =
        get("/api/tags/test-tag-post/posts/?page=1&page_size=10&order_by=-cid").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let count = body.get("count").unwrap().as_u64().unwrap();

    let data = json!({
        "title": "testPostTag",
        "slug": "test-post-tag",
        "created": 1666666666,
        "text": "testText",
        "status": "publish",
        "allowComment": "1",
        "allowPing": "1",
        "allowFeed": "1",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/posts/", data).await;
    assert_eq!(status_code, StatusCode::OK);

    let data = json!({"slug": "test-post-tag",}).to_string();
    let (status_code, _) = admin_post("/api/tags/test-tag-post/posts/", data).await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, body) =
        get("/api/tags/test-tag-post/posts/?page=1&page_size=10&order_by=-cid").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let new_count = body.get("count").unwrap().as_u64().unwrap();
    assert!(new_count > count);
}
