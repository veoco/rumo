use axum::http::StatusCode;
use serde_json::json;

mod common;
use common::{admin_post, admin_patch, get};

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
    })
    .to_string();
    let (status_code, _) = admin_post("/api/pages/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, _) = get("/api/pages/test-page-create").await;
    assert_eq!(status_code, StatusCode::OK);
}

#[tokio::test]
async fn create_then_modify_page_by_slug_success() {
    let data = json!({
        "title": "testPageModify",
        "slug": "test-page-modify",
        "created": 1666666666,
        "text": "testText",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/pages/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, _) = get("/api/pages/test-page-modify").await;
    assert_eq!(status_code, StatusCode::OK);

    let data = json!({
        "title": "testPageModified",
        "slug": "test-page-modified",
        "created": 1666666666,
        "text": "testText",
    })
    .to_string();
    let (status_code, _) = admin_patch("/api/pages/test-page-modify", data).await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, _) = get("/api/pages/test-page-modified").await;
    assert_eq!(status_code, StatusCode::OK);
}

#[tokio::test]
async fn create_then_get_page_field_success() {
    let data = json!({
        "title": "testPageField",
        "slug": "test-page-field",
        "created": 1666666666,
        "text": "testText",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/pages/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, body) = get("/api/pages/test-page-field").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let count = body.get("fields").unwrap().as_array().unwrap().len();
    assert!(count == 0);

    let data = json!({
        "type": "str",
        "str_value": "test-str_feild",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/pages/test-page-field", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let data = json!({
        "type": "int",
        "int_value": 111,
    })
    .to_string();
    let (status_code, _) = admin_post("/api/pages/test-page-field", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let data = json!({
        "type": "float",
        "float_value": 111.111,
    })
    .to_string();
    let (status_code, _) = admin_post("/api/pages/test-page-field", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, body) = get("/api/pages/test-page-field").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let count = body.get("fields").unwrap().as_array().unwrap().len();
    assert!(count == 3);
}
