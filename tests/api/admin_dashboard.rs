use crate::helpers::{spawn_app, assert_redirect_to};

#[tokio::test]
async fn unauthenticated_users_redirected_to_login() {
    let app = spawn_app().await;

    let response = app.get_admin_dashboard().await;
    assert_redirect_to(&response, "/login");
}
