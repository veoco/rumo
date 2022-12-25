use axum::http::StatusCode;
use serde_json::json;

mod common;
use common::{admin_post, get};

#[tokio::test]
async fn create_then_list_pages_success() {
    let (status_code, body) = get("/api/pages/").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let count = body.get("count").unwrap().as_u64().unwrap();

    let data = json!({
        "title": "testPage",
        "slug": "test-page",
        "created": 1666666666,
        "text": "testText",
        "status": "publish",
        "allowComment": "1",
        "allowPing": "1",
        "allowFeed": "1",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/pages/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, body) = get("/api/pages/").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let new_count = body.get("count").unwrap().as_u64().unwrap();
    assert!(new_count > count);
}

#[tokio::test]
async fn create_then_get_page_by_slug_success() {
    let data = json!({
        "title": "testPageCreate",
        "slug": "test-page-create",
        "created": 1666666666,
        "text": "testText",
        "status": "publish",
        "allowComment": "1",
        "allowPing": "1",
        "allowFeed": "1",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/pages/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, _) = get("/api/pages/test-page-create").await;
    assert_eq!(status_code, StatusCode::OK);
}
