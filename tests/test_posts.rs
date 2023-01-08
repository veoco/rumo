use axum::http::StatusCode;
use serde_json::json;

mod common;
use common::{admin_post, admin_patch, admin_delete, get};

#[tokio::test]
async fn create_then_list_posts_success() {
    let (status_code, body) = get("/api/posts/?page=1&page_size=10&order_by=-cid").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let count = body.get("count").unwrap().as_u64().unwrap();

    let data = json!({
        "title": "testPost",
        "slug": "test-post",
        "created": 1666666666,
        "text": "testText",
        "status": "publish",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/posts/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let data = json!({
        "title": "testPost2",
        "slug": "test-post-2",
        "created": 1666666666,
        "text": "testText",
        "status": "publish",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/posts/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let data = json!({
        "title": "testPost3",
        "slug": "test-post-3",
        "created": 1666666666,
        "text": "testText",
        "status": "publish",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/posts/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, body) = get("/api/posts/?page=1&page_size=10&order_by=-cid").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let new_count = body.get("count").unwrap().as_u64().unwrap();
    assert!(new_count > count);
}

#[tokio::test]
async fn create_then_get_post_by_slug_success() {
    let data = json!({
        "title": "testPostCreate",
        "slug": "test-post-create",
        "created": 1666666666,
        "text": "testText",
        "status": "publish",
    })
    .to_string();

    let (status_code, _) = admin_post("/api/posts/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, _) = get("/api/posts/test-post-create").await;
    assert_eq!(status_code, StatusCode::OK);
}

#[tokio::test]
async fn create_then_modify_post_by_slug_success() {
    let data = json!({
        "title": "testPostModify",
        "slug": "test-post-modify",
        "created": 1666666666,
        "text": "testText",
        "status": "publish",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/posts/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, _) = get("/api/posts/test-post-modify").await;
    assert_eq!(status_code, StatusCode::OK);

    let data = json!({
        "title": "testPostModied",
        "slug": "test-post-modified",
        "created": 1666666666,
        "text": "testTextModified",
        "status": "publish",
    })
    .to_string();
    let (status_code, _) = admin_patch("/api/posts/test-post-modify", data).await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, _) = get("/api/posts/test-post-modified").await;
    assert_eq!(status_code, StatusCode::OK);
}

#[tokio::test]
async fn create_then_delete_post_by_slug_success() {
    let data = json!({
        "title": "testPostDelete",
        "slug": "test-post-delete",
        "created": 1666666666,
        "text": "testText",
        "status": "publish",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/posts/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, _) = get("/api/posts/test-post-delete").await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, _) = admin_delete("/api/posts/test-post-delete").await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, _) = get("/api/posts/test-post-delete").await;
    assert_eq!(status_code, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_then_get_post_field_success() {
    let data = json!({
        "title": "testPostField",
        "slug": "test-post-field",
        "created": 1666666666,
        "text": "testText",
        "status": "publish",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/posts/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, body) = get("/api/posts/test-post-field").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let fields = body.get("fields").unwrap().as_array().unwrap();
    assert!(fields.len() == 0);

    let data = json!({
        "name": "test_str",
        "type": "str",
        "str_value": "test-str-feild",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/posts/test-post-field/fields/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let data = json!({
        "name": "test_int",
        "type": "int",
        "int_value": 111,
    })
    .to_string();
    let (status_code, _) = admin_post("/api/posts/test-post-field/fields/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let data = json!({
        "name": "test_float",
        "type": "float",
        "float_value": 111.111,
    })
    .to_string();
    let (status_code, _) = admin_post("/api/posts/test-post-field/fields/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, body) = get("/api/posts/test-post-field").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let count = body.get("fields").unwrap().as_array().unwrap().len();
    assert!(count == 3);
}

#[tokio::test]
async fn create_then_modify_post_field_success() {
    let data = json!({
        "title": "testPostFieldModify",
        "slug": "test-post-field-modify",
        "created": 1666666666,
        "text": "testText",
        "status": "publish",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/posts/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let data = json!({
        "name": "test_str",
        "type": "str",
        "str_value": "test-str-feild",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/posts/test-post-field-modify/fields/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, body) = get("/api/posts/test-post-field-modify/fields/test_str").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let value = body.get("str_value").unwrap().as_str().unwrap();
    assert!(value == "test-str-feild");

    let data = json!({
        "name": "test_str",
        "type": "str",
        "str_value": "test-str-feild-modified",
    })
    .to_string();
    let (status_code, _) = admin_patch("/api/posts/test-post-field-modify/fields/test_str", data).await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, body) = get("/api/posts/test-post-field-modify/fields/test_str").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let value = body.get("str_value").unwrap().as_str().unwrap();
    assert!(value == "test-str-feild-modified");
}

#[tokio::test]
async fn create_then_delete_post_field_success() {
    let data = json!({
        "title": "testPostFieldDelete",
        "slug": "test-post-field-delete",
        "created": 1666666666,
        "text": "testText",
        "status": "publish",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/posts/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, _) = get("/api/posts/test-post-field-delete").await;
    assert_eq!(status_code, StatusCode::OK);

    let data = json!({
        "name": "test_str",
        "type": "str",
        "str_value": "test-str-feild",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/posts/test-post-field-delete/fields/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let data = json!({
        "name": "test_str_2",
        "type": "str",
        "str_value": "test-str-feild-2",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/posts/test-post-field-delete/fields/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, body) = get("/api/posts/test-post-field-delete").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let count = body.get("fields").unwrap().as_array().unwrap().len();
    assert!(count == 2);

    let (status_code, _) = admin_delete("/api/posts/test-post-field-delete/fields/test_str").await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, body) = get("/api/posts/test-post-field-delete").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let count = body.get("fields").unwrap().as_array().unwrap().len();
    assert!(count == 1);
}
