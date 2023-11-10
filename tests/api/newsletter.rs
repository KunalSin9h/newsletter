use std::time::Duration;

use crate::helpers::{assert_redirect_to, spawn_app, ConfirmationLink, TestApp};
use fake::{
    faker::{internet::en::SafeEmail, name::en::Name},
    Fake,
};
use wiremock::{
    matchers::{any, method, path},
    Mock, MockBuilder, ResponseTemplate,
};

#[tokio::test]
async fn newsletter_is_not_delivered_to_unconfirmed_subscribers() {
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    // Because the user is un-confirmed, we should expect 0 request to this
    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    // Send a newsletter,
    let newsletter_request_payload = serde_json::json!({
        "title": "Title of Newsletter",
        "text": "Newsletter plain body",
        "html": "<h1>Newsletter html body</h1>",
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
    });

    let response = app.post_newsletter(&newsletter_request_payload).await;

    assert_redirect_to(&response, "/admin/newsletters");

    app.dispatch_all_emails().await;
}

#[tokio::test]
async fn newsletter_is_delivered_to_confirmed_subscribers() {
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let newsletter_request_payload = serde_json::json!({
        "title": "Title of Newsletter",
        "text": "Newsletter plain body",
        "html": "<h1>Newsletter html body</h1>",
        "idempotency_key": uuid::Uuid::new_v4().to_string(),
    });

    let response = app.post_newsletter(&newsletter_request_payload).await;

    assert_redirect_to(&response, "/admin/newsletters");
    // no email send
    app.dispatch_all_emails().await;
}

async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLink {
    // when working with multiple subscribers
    // use random credentials
    let name: String = Name().fake();
    let email: String = SafeEmail().fake();
    let body = serde_urlencoded::to_string(&serde_json::json!({
        "name": name,
        "email": email
    }))
    .unwrap();

    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;

    app.post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();

    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();

    app.get_confirmation_url(&email_request).await
}

async fn create_confirmed_subscriber(app: &TestApp) {
    let confirmation_link = create_unconfirmed_subscriber(app).await;

    reqwest::get(confirmation_link.html_link)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

#[tokio::test]
async fn publish_newsletter_return_400_for_invalid_body() {
    let app = spawn_app().await;
    app.test_user.login(&app).await;

    let test_case = vec![
        (
            serde_json::json!({
                "text": "",
                "html": "",
            }),
            "missing title",
        ),
        (
            serde_json::json!({
                "title": "",
            }),
            "missing content",
        ),
    ];

    for (content, message) in test_case {
        let res = app.post_newsletter(&content).await;

        assert_eq!(
            res.status().as_u16(),
            400,
            "The API does not fail with 400 Bad Request when payload was {}.",
            message
        );
    }
}

// idempotent == retry-safe
// newsletter creating api = POST /admin/newsletters
#[tokio::test]
async fn newsletter_creation_is_idempotent() {
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    // // mocking Postmark API for testing
    Mock::given(path("/email")) // if request comes in /email
        .and(method("POST")) // with method POST
        .respond_with(ResponseTemplate::new(200))
        .expect(1) // only expect one request
        // when we retry, then this ensures
        // that the api is idempotent
        .mount(&app.email_server) // instance of MockServer
        .await;

    let newsletter_request_payload = serde_json::json!({
        "title": "Newsletter title",
        "text": "text content",
        "html": "<h1>html content</h1>",
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });

    let response = app.post_newsletter(&newsletter_request_payload).await;
    assert_redirect_to(&response, "/admin/newsletters");

    // retrying, i.e submitting the same newsletter **again**
    let response = app.post_newsletter(&newsletter_request_payload).await;
    assert_redirect_to(&response, "/admin/newsletters");

    // mock verifies that the email-client is hit only once
    app.dispatch_all_emails().await;
}

#[tokio::test]
async fn concurrent_form_submission_are_handle_gracefully() {
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    // // mocking Postmark api
    Mock::given(path("/email"))
        .and(method("POST"))
        // setting delay of 2 second for first request to mack the time taken by first request
        // so that we can test the concurrent request being queued
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(2)))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let newsletter_request_payload = serde_json::json!({
        "title": "Newsletter title",
        "text": "text content",
        "html": "<h1>html content</h1>",
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });

    let res_1 = app.post_newsletter(&newsletter_request_payload);
    let res_2 = app.post_newsletter(&newsletter_request_payload);

    let (res_1, res_2) = tokio::join!(res_1, res_2);

    assert_eq!(res_1.status(), res_2.status());
    assert_eq!(res_1.text().await.unwrap(), res_2.text().await.unwrap());

    app.dispatch_all_emails().await;
}

// short-hand for building mock server for email deliveries
fn when_sending_email() -> MockBuilder {
    Mock::given(path("/email")).and(method("POST"))
}

#[tokio::test]
async fn transient_errors_do_not_cause_duplicate_deliveries_on_retry() {
    let app = spawn_app().await;
    let newsletter_request_payload = serde_json::json!({
        "title": "Newsletter title",
        "text": "text content",
        "html": "<h1>html content</h1>",
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });

    create_confirmed_subscriber(&app).await;
    create_confirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    // mocking postmark api
    when_sending_email()
        .respond_with(ResponseTemplate::new(200))
        .up_to_n_times(1)
        .expect(1)
        .mount(&app.email_server)
        .await;

    when_sending_email()
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(1)
        .expect(1)
        .mount(&app.email_server)
        .await;

    let response = app.post_newsletter(&newsletter_request_payload).await;
    assert_eq!(response.status().as_u16(), 303);

    // // retry
    // expecting only one hit to postmark api
    // one subscriber already got the email
    when_sending_email()
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    let response = app.post_newsletter(&newsletter_request_payload).await;
    assert_eq!(response.status().as_u16(), 303);

    app.dispatch_all_emails().await;
}
