use axum::http::StatusCode;
use serde_json::json;

mod common;
use common::{admin_delete, admin_get, admin_post, admin_post_file, get_multipart, admin_patch_file};

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
async fn create_then_get_attachment_success() {
    let data = get_multipart("testFileGet.png", "image/png");
    let (status_code, _) = admin_post_file("/api/attachments/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, body) = admin_get("/api/attachments/").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let attachments = body.get("results").unwrap().as_array().unwrap().clone();
    let mut cid = 0;
    for at in attachments{
        let name = at.get("name").unwrap().as_str().unwrap();
        if name == "testFileGet.png" {
            cid = at.get("cid").unwrap().as_u64().unwrap();
        }
    }

    let url = format!("/api/attachments/{cid}");
    let (status_code, body) = admin_get(&url).await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let name = body.get("name").unwrap().as_str().unwrap();
    assert!(name == "testFileGet.png");
}

#[tokio::test]
async fn create_then_modify_attachment_success() {
    let data = get_multipart("testFileModify.png", "image/png");
    let (status_code, _) = admin_post_file("/api/attachments/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, body) = admin_get("/api/attachments/").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let attachments = body.get("results").unwrap().as_array().unwrap().clone();
    let mut cid = 0;
    for at in attachments{
        let name = at.get("name").unwrap().as_str().unwrap();
        if name == "testFileModify.png" {
            cid = at.get("cid").unwrap().as_u64().unwrap();
        }
    }

    let url = format!("/api/attachments/{cid}");
    let (status_code, body) = admin_get(&url).await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let name = body.get("name").unwrap().as_str().unwrap();
    assert!(name == "testFileModify.png");

    let data = get_multipart("testFileModified.png", "image/png");
    let (status_code, body) = admin_patch_file(&url, data).await;
    println!("{:?}", body);
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, body) = admin_get(&url).await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let name = body.get("name").unwrap().as_str().unwrap();
    assert!(name == "testFileModified.png");
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

#[tokio::test]
async fn create_then_delete_post_attachments_success() {
    let data = get_multipart("testPostFile.png", "image/png");
    let (status_code, _) = admin_post_file("/api/attachments/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, body) = admin_get("/api/attachments/").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let attachments = body.get("results").unwrap().as_array().unwrap().clone();
    let mut cid = 0;
    for at in attachments{
        let name = at.get("name").unwrap().as_str().unwrap();
        if name == "testPostFile.png" {
            cid = at.get("cid").unwrap().as_u64().unwrap();
        }
    }

    let data = json!({
        "title": "testAttachmentPostList",
        "slug": "test-attachment-post-list",
        "created": 1666666666,
        "text": "testText",
        "status": "publish",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/posts/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, body) = admin_get("/api/posts/test-attachment-post-list/attachments/").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let count = body.get("count").unwrap().as_u64().unwrap();
    
    let data = json!({
        "cid": cid,
    })
    .to_string();
    let (status_code, _) = admin_post("/api/posts/test-attachment-post-list/attachments/", data).await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, body) = admin_get("/api/posts/test-attachment-post-list/attachments/").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let new_count = body.get("count").unwrap().as_u64().unwrap();
    assert!(new_count > count);

    let url = format!("/api/posts/test-attachment-post-list/attachments/{cid}");
    let (status_code, _) = admin_delete(&url).await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, body) = admin_get("/api/posts/test-attachment-post-list/attachments/").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let new_count = body.get("count").unwrap().as_u64().unwrap();
    assert!(new_count <= count);
}
