use axum::http::StatusCode;

mod common;
use common::{admin_delete, admin_get, admin_post_file, get_multipart};

#[tokio::test]
async fn create_then_list_attachments_success() {
    let data = get_multipart("testFile.png", "image/png");
    let (status_code, _) = admin_post_file("/api/attachments/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, body) = admin_get("/api/attachments/").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let count = body.get("count").unwrap().as_u64().unwrap();
    assert!(count > 0);
}

#[tokio::test]
async fn create_then_delete_attachments_success() {
    let data = get_multipart("testFile2.png", "image/png");
    let (status_code, _) = admin_post_file("/api/attachments/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, _) = admin_get("/api/attachments/").await;
    assert_eq!(status_code, StatusCode::OK);

    let data = get_multipart("testFile3.png", "image/png");
    let (status_code, _) = admin_post_file("/api/attachments/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, _) = admin_get("/api/attachments/").await;
    assert_eq!(status_code, StatusCode::OK);

    let data = get_multipart("testFile4.png", "image/png");
    let (status_code, _) = admin_post_file("/api/attachments/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let (status_code, body) = admin_get("/api/attachments/").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let count = body.get("count").unwrap().as_u64().unwrap();

    let url = format!("/api/attachments/{}", 3);
    let (status_code, _) = admin_delete(&url).await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, body) = admin_get("/api/attachments/").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let new_count = body.get("count").unwrap().as_u64().unwrap();
    assert!(new_count < count)
}
