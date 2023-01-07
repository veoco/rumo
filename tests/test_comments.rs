use axum::http::StatusCode;
use serde_json::json;

mod common;
use common::{admin_post, admin_get, admin_patch, admin_delete, get, post};

#[tokio::test]
async fn create_then_list_comments_success() {
    let data = json!({
        "title": "testCommentPost",
        "slug": "test-comment-post",
        "created": 1666666666,
        "text": "testText",
        "status": "publish",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/posts/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let data = json!({
        "author": "testAuthor",
        "mail": "test@local.host",
        "url": "https://127.0.0.1",
        "text": "test comment",
    })
    .to_string();
    let (status_code, _) = post("/api/posts/test-comment-post/comments/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, body) = get("/api/posts/test-comment-post/comments/").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let count = body.get("count").unwrap().as_u64().unwrap();
    assert!(count > 0);

    let (status_code, body) = admin_get("/api/comments/").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let count = body.get("count").unwrap().as_u64().unwrap();
    assert!(count > 0);
}

#[tokio::test]
async fn create_then_modify_comments_success() {
    let data = json!({
        "title": "testCommentPostModify",
        "slug": "test-comment-post-modify",
        "created": 1666666666,
        "text": "testText",
        "status": "publish",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/posts/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let data = json!({
        "author": "testAuthor",
        "mail": "test@local.host",
        "url": "https://127.0.0.1",
        "text": "test comment modify",
    })
    .to_string();
    let (status_code, body) = post("/api/posts/test-comment-post-modify/comments/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let body = body.unwrap();
    let coid = body.get("id").unwrap().as_u64().unwrap();

    let url = format!("/api/comments/{}", coid);
    let (status_code, _) = admin_get(&url).await;
    assert_eq!(status_code, StatusCode::OK);

    let data = json!({
        "text": "test comment modified",
        "status": "spam",
    })
    .to_string();
    let (status_code, _) = admin_patch(&url, data).await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, body) = admin_get(&url).await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let status = body.get("status").unwrap().as_str().unwrap();
    assert_eq!(status, "spam");
}

#[tokio::test]
async fn create_then_delete_comments_success() {
    let data = json!({
        "title": "testCommentPostDelete",
        "slug": "test-comment-post-delete",
        "created": 1666666666,
        "text": "testText",
        "status": "publish",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/posts/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let data = json!({
        "author": "testAuthor",
        "mail": "test@local.host",
        "url": "https://127.0.0.1",
        "text": "test comment delete",
    })
    .to_string();
    let (status_code, body) = post("/api/posts/test-comment-post-delete/comments/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let body = body.unwrap();
    let coid = body.get("id").unwrap().as_u64().unwrap();

    let url = format!("/api/comments/{}", coid);
    let (status_code, _) = admin_get(&url).await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, _) = admin_delete(&url).await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, body) = admin_get(&url).await;
    assert_eq!(status_code, StatusCode::NOT_FOUND);
}
