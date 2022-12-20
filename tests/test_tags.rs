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
