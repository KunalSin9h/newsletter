use crate::helpers::spawn_app;

#[tokio::test]
pub async fn confirmation_without_token_will_give_400_error() {
    let test_app = spawn_app().await;

    let res = reqwest::get(&format!("{}/subscription/confirm", test_app.address))
        .await
        .unwrap();

    assert_eq!(res.status(), 400);
}
