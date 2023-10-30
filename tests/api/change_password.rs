use std::collections::HashMap;

use crate::helpers::{assert_redirect_to, spawn_app};

#[tokio::test]
async fn user_must_be_logged_in_to_see_change_password_form() {
    let app = spawn_app().await;

    let response = app.get_change_password().await;

    assert_redirect_to(&response, "/login");
}

#[tokio::test]
async fn user_must_be_logged_in_to_change_password() {
    let app = spawn_app().await;
    let new_password = uuid::Uuid::new_v4().to_string();

    let mut request_payload = HashMap::new();
    request_payload.insert("current_password", uuid::Uuid::new_v4().to_string());
    request_payload.insert("new_password", new_password.clone());
    request_payload.insert("new_password_check", new_password);

    let response = app.post_change_password(&request_payload).await;

    assert_redirect_to(&response, "/login");
}

#[tokio::test]
async fn new_password_fields_password_must_match() {
    let app = spawn_app().await;

    let mut login_request_payload = HashMap::new();
    login_request_payload.insert("username", &app.test_user.username);
    login_request_payload.insert("password", &app.test_user.password);

    app.post_login(&login_request_payload).await;

    let new_password = uuid::Uuid::new_v4().to_string();
    let another_new_password = uuid::Uuid::new_v4().to_string();

    let mut change_password_request_payload = HashMap::new();
    change_password_request_payload.insert("current_password", &app.test_user.password);
    change_password_request_payload.insert("new_password", &new_password);
    change_password_request_payload.insert("new_password_check", &another_new_password);

    let response = app
        .post_change_password(&change_password_request_payload)
        .await;

    assert_redirect_to(&response, "/admin/password");

    let html_page = app.get_change_password_html().await;

    assert!(html_page
        .contains("You entered two different new passwords - the field values must match."));
}

#[tokio::test]
async fn current_password_should_be_valid() {
    let app = spawn_app().await;

    let mut login_request_payload = HashMap::new();
    login_request_payload.insert("username", &app.test_user.username);
    login_request_payload.insert("password", &app.test_user.password);

    app.post_login(&login_request_payload).await;

    let new_password = uuid::Uuid::new_v4().to_string();
    let wrong_password = uuid::Uuid::new_v4().to_string();

    let mut change_password_request_payload = HashMap::new();
    change_password_request_payload.insert("current_password", &wrong_password);
    change_password_request_payload.insert("new_password", &new_password);
    change_password_request_payload.insert("new_password_check", &new_password);

    let response = app
        .post_change_password(&change_password_request_payload)
        .await;

    assert_redirect_to(&response, "/admin/password");

    let html_page = app.get_change_password_html().await;

    assert!(html_page.contains("The current password is incorrect."));
}
