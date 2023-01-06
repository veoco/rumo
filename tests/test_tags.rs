use axum::http::StatusCode;
use serde_json::json;

mod common;
use common::{admin_post, admin_delete, admin_patch, get};

#[tokio::test]
async fn create_then_list_tags_success() {
    let (status_code, body) = get("/api/tags/").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let count = body.get("count").unwrap().as_u64().unwrap();

    let data = json!({"name": "testTag", "slug": "test-tag"}).to_string();
    let (status_code, _) = admin_post("/api/tags/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let data = json!({"name": "testTag2", "slug": "test-tag-2"}).to_string();
    let (status_code, _) = admin_post("/api/tags/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let data = json!({"name": "testTag3", "slug": "test-tag-3"}).to_string();
    let (status_code, _) = admin_post("/api/tags/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, body) = get("/api/tags/").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let new_count = body.get("count").unwrap().as_u64().unwrap();
    assert!(new_count > count);
}

#[tokio::test]
async fn create_then_get_tag_by_slug_success() {
    let data = json!({"name": "testTagCreate", "slug": "test-tag-create"}).to_string();
    let (status_code, _) = admin_post("/api/tags/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, _) = get("/api/tags/test-tag-create").await;
    assert_eq!(status_code, StatusCode::OK);
}

#[tokio::test]
async fn create_then_modify_tag_by_slug_success() {
    let data = json!({"name": "testTagModify", "slug": "test-tag-modify"}).to_string();
    let (status_code, _) = admin_post("/api/tags/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, _) = get("/api/tags/test-tag-modify").await;
    assert_eq!(status_code, StatusCode::OK);

    let data =
        json!({"name": "testTagModified", "slug": "test-tag-modified"}).to_string();
    let (status_code, _) = admin_patch("/api/tags/test-tag-modify", data).await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, _) = get("/api/tags/test-tag-modified").await;
    assert_eq!(status_code, StatusCode::OK);
}

#[tokio::test]
async fn create_then_delete_tag_by_slug_success() {
    let data = json!({"name": "testTagDelete", "slug": "test-tag-delete"}).to_string();
    let (status_code, _) = admin_post("/api/tags/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, _) = get("/api/tags/test-tag-delete").await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, _) = admin_delete("/api/tags/test-tag-delete").await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, _) = get("/api/tags/test-tag-delete").await;
    assert_eq!(status_code, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_then_list_tag_posts_success() {
    let data = json!({"name": "testTagPost", "slug": "test-tag-post"}).to_string();
    let (status_code, _) = admin_post("/api/tags/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, body) = get("/api/tags/test-tag-post/posts/").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let count = body.get("count").unwrap().as_u64().unwrap();

    let data = json!({
        "title": "testPostTag",
        "slug": "test-post-tag",
        "created": 1666666666,
        "text": "testText",
        "status": "publish",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/posts/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let data = json!({"slug": "test-post-tag",}).to_string();
    let (status_code, _) = admin_post("/api/tags/test-tag-post/posts/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, body) = get("/api/tags/test-tag-post/posts/").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let new_count = body.get("count").unwrap().as_u64().unwrap();
    assert!(new_count > count);
}

#[tokio::test]
async fn create_then_delete_tag_posts_success() {
    let data = json!({"name": "testTagPostDelete", "slug": "test-tag-post-delete"}).to_string();
    let (status_code, _) = admin_post("/api/tags/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let data = json!({
        "title": "testPostTagDelete",
        "slug": "test-post-tag-delete",
        "created": 1666666666,
        "text": "testText",
        "status": "publish",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/posts/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let data = json!({"slug": "test-post-tag-delete",}).to_string();
    let (status_code, _) = admin_post("/api/tags/test-tag-post-delete/posts/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, body) = get("/api/tags/test-tag-post-delete/posts/").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let count = body.get("count").unwrap().as_u64().unwrap();

    let (status_code, _) = admin_delete("/api/tags/test-tag-post-delete/posts/test-post-tag-delete").await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, body) = get("/api/tags/test-tag-post-delete/posts/").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let new_count = body.get("count").unwrap().as_u64().unwrap();

    assert!(new_count < count);
}
