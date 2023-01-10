use axum::http::StatusCode;
use serde_json::json;

mod common;
use common::{admin_post, admin_patch, get, admin_delete};

#[tokio::test]
async fn create_then_list_pages_success() {
    let (status_code, body) = get("/api/pages/").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let count = body.get("all_count").unwrap().as_u64().unwrap();

    let data = json!({
        "title": "testPage",
        "slug": "test-page",
        "created": 1666666666,
        "text": "testText",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/pages/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let data = json!({
        "title": "testPage2",
        "slug": "test-page-2",
        "created": 1666666666,
        "text": "testText",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/pages/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let data = json!({
        "title": "testPage3",
        "slug": "test-page-3",
        "created": 1666666666,
        "text": "testText",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/pages/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, body) = get("/api/pages/").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let new_count = body.get("all_count").unwrap().as_u64().unwrap();
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
    println!("{:?}", body);
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
    let (status_code, _) = admin_post("/api/pages/test-page-field/fields/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let data = json!({
        "name": "test_int",
        "type": "int",
        "int_value": 111,
    })
    .to_string();
    let (status_code, _) = admin_post("/api/pages/test-page-field/fields/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let data = json!({
        "name": "test_float",
        "type": "float",
        "float_value": 111.111,
    })
    .to_string();
    let (status_code, _) = admin_post("/api/pages/test-page-field/fields/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, body) = get("/api/pages/test-page-field").await;
    println!("{:?}", body);
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let count = body.get("fields").unwrap().as_array().unwrap().len();
    assert!(count == 3);
}

#[tokio::test]
async fn create_then_modify_page_field_success() {
    let data = json!({
        "title": "testPageFieldModify",
        "slug": "test-page-field-modify",
        "created": 1666666666,
        "text": "testText",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/pages/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let data = json!({
        "name": "test_str",
        "type": "str",
        "str_value": "test-str-feild",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/pages/test-page-field-modify/fields/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, body) = get("/api/pages/test-page-field-modify/fields/test_str").await;
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
    let (status_code, _) = admin_patch("/api/pages/test-page-field-modify/fields/test_str", data).await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, body) = get("/api/pages/test-page-field-modify/fields/test_str").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let value = body.get("str_value").unwrap().as_str().unwrap();
    assert!(value == "test-str-feild-modified");
}

#[tokio::test]
async fn create_then_delete_page_field_success() {
    let data = json!({
        "title": "testPageFieldDelete",
        "slug": "test-page-field-delete",
        "created": 1666666666,
        "text": "testText",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/pages/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, _) = get("/api/pages/test-page-field-delete").await;
    assert_eq!(status_code, StatusCode::OK);

    let data = json!({
        "name": "test_str",
        "type": "str",
        "str_value": "test-str-feild",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/pages/test-page-field-delete/fields/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let data = json!({
        "name": "test_str_2",
        "type": "str",
        "str_value": "test-str-feild-2",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/pages/test-page-field-delete/fields/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, body) = get("/api/pages/test-page-field-delete").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let count = body.get("fields").unwrap().as_array().unwrap().len();
    assert!(count == 2);

    let (status_code, _) = admin_delete("/api/pages/test-page-field-delete/fields/test_str").await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, body) = get("/api/pages/test-page-field-delete").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let count = body.get("fields").unwrap().as_array().unwrap().len();
    assert!(count == 1);
}

#[tokio::test]
async fn create_then_delete_page_success() {
    let data = json!({
        "title": "testPageDelete",
        "slug": "test-page-delete",
        "created": 1666666666,
        "text": "testText",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/pages/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, _) = get("/api/pages/test-page-delete").await;
    assert_eq!(status_code, StatusCode::OK);

    let data = json!({
        "name": "test_str",
        "type": "str",
        "str_value": "test-str-feild",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/pages/test-page-delete/fields/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let data = json!({
        "name": "test_str_2",
        "type": "str",
        "str_value": "test-str-feild-2",
    })
    .to_string();
    let (status_code, _) = admin_post("/api/pages/test-page-delete/fields/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);


    let (status_code, _) = admin_delete("/api/pages/test-page-delete").await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, _) = get("/api/pages/test-page-delete").await;
    assert_eq!(status_code, StatusCode::NOT_FOUND);

    let (status_code, _) = get("/api/pages/test-page-delete/fields/test_str").await;
    assert_eq!(status_code, StatusCode::NOT_FOUND);
}
