use crate::helpers::{assert_redirect_to, spawn_app};
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn subscribe_redirect_for_valid_form_data() {
    let test_app = spawn_app().await;

    let body = "name=Kunal%20Singh&email=kunal%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    let response = test_app.post_subscriptions(body.to_string()).await;

    assert_eq!(303, response.status().as_u16());
    assert_redirect_to(&response, "/subscription");
}

#[tokio::test]
async fn subscribe_persist_the_new_subscriber() {
    let test_app = spawn_app().await;

    let body = "name=Kunal%20Singh&email=kunal%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    let response = test_app.post_subscriptions(body.to_string()).await;

    assert_eq!(303, response.status().as_u16());
    assert_redirect_to(&response, "/subscription");

    let saved = sqlx::query!("SELECT email, name, status from subscriptions",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "kunal@gmail.com");
    assert_eq!(saved.name, "Kunal Singh");
    assert_eq!(saved.status, "pending_confirmation");
}

#[tokio::test]
async fn subscribe_return_a_400_when_data_is_missing() {
    let test_app = spawn_app().await;

    let test_cases = vec![
        ("name=Kunal%20Singh", "missing email in the body"),
        ("email=kunal%40gmail.com", "missing name in the body"),
        ("", "missing both(name & email) in the body"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = test_app.post_subscriptions(invalid_body.to_string()).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API does not fail with 400 BAD REQUEST when the payload is {}",
            error_message
        )
    }
}

#[tokio::test]
async fn subscribe_return_a_400_when_fields_are_present_but_empty() {
    let test_app = spawn_app().await;

    let test_cases = vec![
        ("name=&email=a%40b.com", "missing name value"),
        ("name=a&email=", "missing email value"),
        ("name=a&email=definitely_not_an_email", "missing email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = test_app.post_subscriptions(invalid_body.to_string()).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "the api didn't respond with 400 when the payload was {}",
            error_message
        )
    }
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_when_valid_data() {
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
    let confirmation_links = test_app.get_confirmation_url(email_request).await;
    assert_eq!(confirmation_links.html_link, confirmation_links.text_link);
}

#[tokio::test]
async fn subscribe_fails_if_there_if_fatal_db_error() {
    let test_app = spawn_app().await;
    let body = "name=Test%20%Test&email=Test%40email.com";

    sqlx::query!("ALTER TABLE subscription_tokens DROP COLUMN subscription_token;",)
        .execute(&test_app.db_pool)
        .await
        .unwrap();

    let response = test_app.post_subscriptions(body.into()).await;

    assert_eq!(response.status().as_u16(), 500);
}
