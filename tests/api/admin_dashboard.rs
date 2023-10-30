use crate::helpers::{assert_redirect_to, spawn_app};

#[tokio::test]
async fn unauthenticated_users_redirected_to_login() {
    let app = spawn_app().await;

    let response = app.get_admin_dashboard().await;
    assert_redirect_to(&response, "/login");
}

#[tokio::test]
async fn logout_clears_session_state() {
    let app = spawn_app().await;

    let _response = app
        .post_login(&serde_json::json!({
            "username": &app.test_user.username,
            "password": &app.test_user.password,
        }))
        .await;

    let logout_res = app.post_logout().await;
    assert_redirect_to(&logout_res, "/login");

    let login_page_html = app.get_login_html().await;
    assert!(login_page_html.contains(r#"You have successfully logged out."#));

    let dashboard_res = app.get_admin_dashboard().await;
    assert_redirect_to(&dashboard_res, "/login");
}
