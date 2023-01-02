use axum::http::StatusCode;
use serde_json::json;

mod common;
use common::{admin_post, admin_get, get, post};

#[tokio::test]
async fn create_then_list_comments_success() {
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
    println!("{:?}", body);
    let count = body.get("count").unwrap().as_u64().unwrap();
    assert!(count > 0);
}
