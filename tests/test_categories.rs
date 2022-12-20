use axum::http::StatusCode;
use serde_json::json;

mod common;
use common::{admin_post, get};

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
