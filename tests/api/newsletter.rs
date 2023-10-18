use crate::helpers::{spawn_app, ConfirmationLink, TestApp};
use wiremock::{
    matchers::{any, method, path},
    Mock, ResponseTemplate,
};

#[tokio::test]
async fn newsletter_is_not_delivered_to_unconfirmed_subscribers() {
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;

    // Because the user is un-confirmed, we should expect 0 request to this
    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    // Send a newsletter,
    let newsletter_request_payload = serde_json::json!({
        "title": "Title of Newsletter",
        "content": {
            "text": "Newsletter plain body",
            "html": "<h1>Newsletter html body</h1>",
        }
    });

    let response = app.post_newsletter(newsletter_request_payload).await;

    assert_eq!(response.status().as_u16(), 200);
}

#[tokio::test]
async fn newsletter_is_delivered_to_confirmed_subscribers() {
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    let newsletter_request_payload = serde_json::json!({
        "title": "Title of Newsletter",
        "content": {
            "text": "Newsletter plain body",
            "html": "<h1>Newsletter html body</h1>",
        }
    });

    let response = app.post_newsletter(newsletter_request_payload).await;

    assert_eq!(response.status().as_u16(), 200);
}

async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLink {
    let body = "name=test%20test&email=test%40gmail.com";

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
    let test_case = vec![
        (
            serde_json::json!({
                "content": {
                    "text": "",
                    "html": "",
                }
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
        let res = app.post_newsletter(content).await;

        assert_eq!(
            res.status().as_u16(),
            400,
            "The API does not fail with 400 BAT REQUEST when payload was {}.",
            message
        );
    }
}

#[tokio::test]
async fn request_with_missing_authorization_header_must_be_rejected() {
    let app = spawn_app().await;

    let response = reqwest::Client::new()
        .post(format!("{}/newsletters", &app.address))
        .json(&serde_json::json!({
            "title": "Newsletter Title",
            "content": {
                "text": "Text content",
                "html": "<h1>Html content</h1>",
            }
        }))
        .send()
        .await
        .expect("failed to exec request");

    assert_eq!(response.status().as_u16(), 401);
    assert_eq!(
        response.headers()["WWW-Authenticate"],
        r#"Basic realm="publish""#
    );
}

#[tokio::test]
async fn non_existing_user_is_rejected() {
    // Arrange
    let app = spawn_app().await;

    let username = uuid::Uuid::new_v4();
    let password = uuid::Uuid::new_v4();

    let response = reqwest::Client::new()
        .post(format!("{}/newsletters", &app.address))
        .basic_auth(username, Some(password))
        .json(&serde_json::json!({
            "title": "Newsletter",
            "content": {
                "text": "Newsletter",
                "html": "Newsletter",
            }
        }))
        .send()
        .await
        .expect("Failed to execute the request");

    assert_eq!(401, response.status().as_u16());
    assert_eq!(
        r#"Basic realm="publish""#,
        response.headers()["WWW-Authenticate"]
    )
}

#[tokio::test]
async fn user_with_wrong_password_is_rejected() {
    // Arrange
    let app = spawn_app().await;

    let username = app.test_user.username;
    let password = uuid::Uuid::new_v4();

    assert_ne!(password.to_string(), app.test_user.password);

    let response = reqwest::Client::new()
        .post(format!("{}/newsletters", &app.address))
        .basic_auth(username, Some(password))
        .json(&serde_json::json!({
            "title": "Newsletter",
            "content": {
                "text": "Newsletter",
                "html": "Newsletter",
            }
        }))
        .send()
        .await
        .expect("Failed to execute the request");

    assert_eq!(401, response.status().as_u16());
    assert_eq!(
        r#"Basic realm="publish""#,
        response.headers()["WWW-Authenticate"]
    )
}
