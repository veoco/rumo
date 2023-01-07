use axum::http::StatusCode;
use serde_json::json;

mod common;
use common::{admin_get, admin_patch, admin_delete, get, post};

#[tokio::test]
async fn index() {
    let (status_code, _) = get("/").await;
    assert_eq!(status_code, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn login_failed() {
    let data = json!({"mail": "login_failed@test.local", "password": "password"}).to_string();
    let (status_code, _) = post("/api/users/token", data).await;
    assert_eq!(status_code, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_success() {
    let data = json!({"name": "login_test","mail": "login_success@test.local", "url": "http://127.0.0.1", "password": "password"}).to_string();
    let (status_code, _) = post("/api/users", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let data = json!({"mail": "login_success@test.local", "password": "password"}).to_string();
    let (status_code, _) = post("/api/users/token", data).await;
    assert_eq!(status_code, StatusCode::OK);
}

#[tokio::test]
async fn normal_user_change_success() {
    let data = json!({"name": "change_test","mail": "change_test@test.local", "url": "http://127.0.0.1", "password": "password"}).to_string();
    let (_, body) = post("/api/users", data).await;
    let body = body.unwrap();
    let uid = body.get("id");
    assert!(uid.is_some());
    let uid = uid.unwrap().as_u64().unwrap();

    let url = format!("/api/users/{}", uid);
    let data = json!({"name": "changed_test", "mail": "changed_test@test.local", "url": "http://127.0.0.1", "screenName": "changed_test", "group": "subscriber"}).to_string();
    let (status_code, _) = admin_patch(&url, data).await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, body) = admin_get(&url).await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let name = body.get("name");
    assert!(name.is_some());

    let name = name.unwrap().as_str().unwrap();
    assert_eq!(name, "changed_test");
}

#[tokio::test]
async fn list_users_success() {
    let (status_code, _) = admin_get("/api/users/").await;
    assert_eq!(status_code, StatusCode::OK);
}

#[tokio::test]
async fn create_then_delete_user_success() {
    let data = json!({"name": "delete_test","mail": "delete_test@test.local", "url": "http://127.0.0.1", "password": "password"}).to_string();
    let (status_code, _) = post("/api/users", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let data = json!({"name": "delete_test2","mail": "delete2_test@test.local", "url": "http://127.0.0.1", "password": "password"}).to_string();
    let (status_code, _) = post("/api/users", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let data = json!({"name": "delete_test3","mail": "delete3_test@test.local", "url": "http://127.0.0.1", "password": "password"}).to_string();
    let (status_code, body) = post("/api/users", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let body = body.unwrap();
    let uid = body.get("id").unwrap().as_u64().unwrap();

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let (status_code, body) = admin_get("/api/users/").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let count = body.get("count").unwrap().as_u64().unwrap();

    let url = format!("/api/users/{uid}");
    let (status_code, _) = admin_delete(&url).await;
    assert_eq!(status_code, StatusCode::OK);

    
    let (status_code, body) = admin_get("/api/users/").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let new_count = body.get("count").unwrap().as_u64().unwrap();
    assert!(new_count < count);
}
