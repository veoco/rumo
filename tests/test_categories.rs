use axum::http::StatusCode;
use serde_json::json;

mod common;
use common::{admin_post, get};
use tracing_subscriber::fmt::format;

#[tokio::test]
async fn create_then_list_categories_success() {
    let (status_code, body) = get("/api/categories/?page=1&page_size=10&order_by=-mid").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let count = body.get("count").unwrap().as_u64().unwrap();

    let data = json!({"name": "testCategory", "slug": "test-category"}).to_string();
    let (status_code, _) = admin_post("/api/categories/", data).await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, body) = get("/api/categories/?page=1&page_size=10&order_by=-mid").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let new_count = body.get("count").unwrap().as_u64().unwrap();
    assert!(new_count > count);
}

#[tokio::test]
async fn create_then_get_category_by_slug_success() {
    let data = json!({"name": "testCategoryCreate", "slug": "test-category-create"}).to_string();
    let (status_code, _) = admin_post("/api/categories/", data).await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, _) = get("/api/categories/test-category-create").await;
    assert_eq!(status_code, StatusCode::OK);
}

#[tokio::test]
async fn create_then_list_category_posts_success() {
    let data = json!({"name": "testCategoryPost", "slug": "test-category-post"}).to_string();
    let (status_code, _) = admin_post("/api/categories/", data).await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, body) = get("/api/categories/test-category-post/posts/?page=1&page_size=10&order_by=-cid").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let count = body.get("count").unwrap().as_u64().unwrap();

    let data = json!({
        "title": "testPostCategory",
        "slug": "test-post-category",
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

    let data = json!({"slug": "test-post-category",}).to_string();
    let (status_code, _) = admin_post("/api/categories/test-category-post/posts/", data).await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, body) = get("/api/categories/test-category-post/posts/?page=1&page_size=10&order_by=-cid").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let new_count = body.get("count").unwrap().as_u64().unwrap();
    assert!(new_count > count);
}
