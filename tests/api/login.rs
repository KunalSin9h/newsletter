use std::collections::HashMap;

use crate::helpers::{assert_redirect_to, spawn_app};

#[tokio::test]
async fn an_error_flash_message_is_set_on_failure() {
    let app = spawn_app().await;

    let mut form_data = HashMap::new();
    form_data.insert("username", "random-username");
    form_data.insert("password", "random-password");

    let response = app.post_login(&form_data).await;

    assert_redirect_to(&response, "/login");

    let response_html = app.get_login_html().await;
    assert!(response_html.contains(r#"Authentication failed"#)); // True

    // Reload the page
    // i.e requesting to same /login, this time the cookie must
    // be vanished
    let response_html = app.get_login_html().await;
    assert!(!response_html.contains(r#"Authentication failed"#)); // False
}

#[tokio::test]
async fn redirect_to_admin_dashboard_on_login_success() {
    let app = spawn_app().await;

    let mut form_data = HashMap::new();
    form_data.insert("username", &app.test_user.username);
    form_data.insert("password", &app.test_user.password);

    let response = app.post_login(&form_data).await;
    assert_redirect_to(&response, "/admin/dashboard");

    let response_html = app.get_admin_dashboard_html().await;
    assert!(response_html.contains(&format!("Welcome {}!", app.test_user.username)));
    // True
}
