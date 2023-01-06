use axum::http::StatusCode;
use serde_json::json;

mod common;
use common::{admin_delete, admin_patch, admin_post, get};

#[tokio::test]
async fn create_then_list_categories_success() {
    let (status_code, body) = get("/api/categories/").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let count = body.get("count").unwrap().as_u64().unwrap();

    let data = json!({"name": "testCategory", "slug": "test-category"}).to_string();
    let (status_code, _) = admin_post("/api/categories/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, body) = get("/api/categories/").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let new_count = body.get("count").unwrap().as_u64().unwrap();
    assert!(new_count > count);
}

#[tokio::test]
async fn create_then_get_category_by_slug_success() {
    let data = json!({"name": "testCategoryCreate", "slug": "test-category-create"}).to_string();
    let (status_code, _) = admin_post("/api/categories/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, _) = get("/api/categories/test-category-create").await;
    assert_eq!(status_code, StatusCode::OK);
}

#[tokio::test]
async fn create_then_modify_category_by_slug_success() {
    let data = json!({"name": "testCategoryModify", "slug": "test-category-modify"}).to_string();
    let (status_code, _) = admin_post("/api/categories/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, _) = get("/api/categories/test-category-modify").await;
    assert_eq!(status_code, StatusCode::OK);

    let data =
        json!({"name": "testCategoryModified", "slug": "test-category-modified"}).to_string();
    let (status_code, _) = admin_patch("/api/categories/test-category-modify", data).await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, _) = get("/api/categories/test-category-modified").await;
    assert_eq!(status_code, StatusCode::OK);
}

#[tokio::test]
async fn create_then_delete_category_by_slug_success() {
    let data = json!({"name": "testCategoryDelete", "slug": "test-category-delete"}).to_string();
    let (status_code, _) = admin_post("/api/categories/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, _) = get("/api/categories/test-category-delete").await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, _) = admin_delete("/api/categories/test-category-delete").await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, _) = get("/api/categories/test-category-delete").await;
    assert_eq!(status_code, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_then_list_category_posts_success() {
    let data = json!({"name": "testCategoryPost", "slug": "test-category-post"}).to_string();
    let (status_code, _) = admin_post("/api/categories/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, body) = get("/api/categories/test-category-post/posts/").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let count = body.get("count").unwrap().as_u64().unwrap();

    let data = json!({
        "title": "testPostCategory",
        "slug": "test-post-category",
        "created": 1666666666,
        "text": "testText",
        "status": "publish",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/posts/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let data = json!({"slug": "test-post-category",}).to_string();
    let (status_code, _) = admin_post("/api/categories/test-category-post/posts/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, body) = get("/api/categories/test-category-post/posts/").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let new_count = body.get("count").unwrap().as_u64().unwrap();
    assert!(new_count > count);
}

#[tokio::test]
async fn create_then_delete_category_post_success() {
    let data =
        json!({"name": "testCategoryPostDelete", "slug": "test-category-post-delete"}).to_string();
    let (status_code, _) = admin_post("/api/categories/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, _) = get("/api/categories/test-category-post-delete/posts/").await;
    assert_eq!(status_code, StatusCode::OK);

    let data = json!({
        "title": "testPostCategoryDelete",
        "slug": "test-post-category-delete",
        "created": 1666666666,
        "text": "testText",
        "status": "publish",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/posts/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let data = json!({"slug": "test-post-category-delete",}).to_string();
    let (status_code, _) =
        admin_post("/api/categories/test-category-post-delete/posts/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, body) = get("/api/categories/test-category-post-delete/posts/").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let count = body.get("count").unwrap().as_u64().unwrap();

    let (status_code, _) =
        admin_delete("/api/categories/test-category-post-delete/posts/test-post-category-delete")
            .await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, body) = get("/api/categories/test-category-post-delete/posts/").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let new_count = body.get("count").unwrap().as_u64().unwrap();
    assert!(new_count < count);
}
