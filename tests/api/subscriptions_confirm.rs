use crate::helpers::spawn_app;
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
pub async fn confirmation_without_token_will_give_400_error() {
    let test_app = spawn_app().await;

    let res = reqwest::get(&format!("{}/subscription/confirm", test_app.address))
        .await
        .unwrap();

    assert_eq!(res.status(), 400);
}

#[tokio::test]
pub async fn the_link_returned_by_subscribe_will_give_200_when_hit() {
    let test_app = spawn_app().await;
    let body = "name=Test%20%Test&email=Test%40email.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    test_app.post_subscriptions(body.into()).await;

    let email_request = &test_app.email_server.received_requests().await.unwrap()[0];
    let confirmation_link = test_app.get_confirmation_url(email_request).await;

    let res = reqwest::get(confirmation_link.text_link).await.unwrap();

    assert_eq!(res.status(), 200);
}

#[tokio::test]
pub async fn clicking_ok_confirmation_link_confirms_a_subscriber() {
    let test_app = spawn_app().await;
    let name = "A%20B";
    let email = "AB%40email.com";
    let body = format!("{}&{}", &name, &email);

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    test_app.post_subscriptions(body.into()).await;

    let email_request = &test_app.email_server.received_requests().await.unwrap()[0];
    let confirmation_link = test_app.get_confirmation_url(email_request).await;

    reqwest::get(confirmation_link.html_link)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    let saved = sqlx::query!("SELECT email, name, status from subscriptions",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, email);
    assert_eq!(saved.name, name);
    assert_eq!(saved.status, "confirmed");
}
