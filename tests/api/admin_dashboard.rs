use crate::helpers::{assert_redirect_to, spawn_app};

#[tokio::test]
async fn unauthenticated_users_redirected_to_login() {
    let app = spawn_app().await;

    let response = app.get_admin_dashboard().await;
    assert_redirect_to(&response, "/login");
}
