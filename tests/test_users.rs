use axum::http::StatusCode;
use serde_json::json;

mod common;
use common::{admin_delete, admin_get, admin_patch, get, post, admin_post};

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
    let (status_code, _) = post("/api/users/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let data = json!({"mail": "login_success@test.local", "password": "password"}).to_string();
    let (status_code, _) = post("/api/users/token", data).await;
    assert_eq!(status_code, StatusCode::OK);
}

#[tokio::test]
async fn normal_user_change_success() {
    let data = json!({"name": "change_test","mail": "change_test@test.local", "url": "http://127.0.0.1", "password": "password"}).to_string();
    let (status_code, _) = post("/api/users/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, body) = admin_get("/api/users/").await;
    assert_eq!(status_code, StatusCode::OK);

    let mut uid = 0;
    let body = body.unwrap();
    let users = body.get("results").unwrap().as_array().unwrap();
    for user in users {
        let name = user.get("name").unwrap().as_str().unwrap();
        if name == "change_test" {
            uid = user.get("uid").unwrap().as_u64().unwrap();
        }
    }
    assert!(uid != 0);

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
    let (status_code, _) = post("/api/users/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let data = json!({"name": "delete_test2","mail": "delete2_test@test.local", "url": "http://127.0.0.1", "password": "password"}).to_string();
    let (status_code, _) = post("/api/users/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let data = json!({"name": "delete_test3","mail": "delete3_test@test.local", "url": "http://127.0.0.1", "password": "password"}).to_string();
    let (status_code, _) = post("/api/users/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let (status_code, body) = admin_get("/api/users/").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let users = body.get("results").unwrap().as_array().unwrap();
    let mut uid = 0;
    for user in users{
        let name = user.get("name").unwrap().as_str().unwrap();
        if name == "delete_test3" {
            uid = user.get("uid").unwrap().as_u64().unwrap();
        }
    }

    let url = format!("/api/users/{}", uid);
    let (status_code, _) = admin_delete(&url).await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, _) = admin_get(&url).await;
    assert_eq!(status_code, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_then_list_user_options_success() {
    let data = json!({"name": "list_option","value": "option_value"}).to_string();
    let (status_code, body) = admin_post("/api/users/1/options/", data).await;
    println!("{:?}", body);
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, body) = admin_get("/api/users/1/options/").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let count = body.get("all_count").unwrap().as_u64().unwrap();
    assert!(count > 0);
}

#[tokio::test]
async fn create_then_modify_user_option_success() {
    let data = json!({"name": "modify_option","value": "modify"}).to_string();
    let (status_code, _) = admin_post("/api/users/1/options/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, _) = admin_get("/api/users/1/options/modify_option").await;
    assert_eq!(status_code, StatusCode::OK);

    let data = json!({"value": "modified"}).to_string();
    let (status_code, body) = admin_patch("/api/users/1/options/modify_option", data).await;
    println!("{:?}", body);
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, body) = admin_get("/api/users/1/options/modify_option").await;
    assert_eq!(status_code, StatusCode::OK);

    let body = body.unwrap();
    let value = body.get("value").unwrap().as_str().unwrap();
    assert!(value == "modified")
}

#[tokio::test]
async fn create_then_delete_user_option_success() {
    let data = json!({"name": "delete_option","value": "option_value"}).to_string();
    let (status_code, _) = admin_post("/api/users/1/options/", data).await;
    assert_eq!(status_code, StatusCode::CREATED);

    let (status_code, _) = admin_get("/api/users/1/options/delete_option").await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, _) = admin_delete("/api/users/1/options/delete_option").await;
    assert_eq!(status_code, StatusCode::OK);

    let (status_code, _) = admin_get("/api/users/1/options/delete_option").await;
    assert_eq!(status_code, StatusCode::NOT_FOUND);
}
