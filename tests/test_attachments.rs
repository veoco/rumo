use axum::http::StatusCode;
use serde_json::json;

mod common;
use common::{admin_post, get, post};

#[tokio::test]
async fn create_then_list_attachments_success() {
    let data = json!({
        "title": "testCommentPost",
        "slug": "test-comment-post",
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

    let data = json!({
        "author": "testAuthor",
        "mail": "test@local.host",
        "url": "https://127.0.0.1",
        "text": "test comment",
    })
    .to_string();
    let (status_code, _) = post("/api/comments/test-comment-post", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, body) = get("/api/comments/test-comment-post").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let count = body.get("count").unwrap().as_u64().unwrap();
    assert!(count > 0);
}
