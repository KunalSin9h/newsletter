use crate::helpers::spawn_app;

#[tokio::test]
async fn subscribe_return_a_200_for_valid_form_data() {
    let test_app = spawn_app().await;

    let client = reqwest::Client::new();

    let body = "name=Kunal%20Singh&email=kunal%40gmail.com";

    let response = client
        .post(format!("{}/subscription", &test_app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name from subscriptions",)
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "kunal@gmail.com");
    assert_eq!(saved.name, "Kunal Singh");
}

#[tokio::test]
async fn subscribe_return_a_400_when_data_is_missing() {
    let test_app = spawn_app().await;

    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=Kunal%20Singh", "missing email in the body"),
        ("email=kunal%40gmail.com", "missing name in the body"),
        ("", "missing both(name & email) in the body"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{}/subscription", test_app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API does not fail with 400 BAD REQUEST when the payload is {}",
            error_message
        )
    }
}

#[tokio::test]
async fn subscribe_return_a_200_when_fields_are_present_but_empty() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=&email=a%40b.com", "missing name value"),
        ("name=a&email=", "missing email value"),
        ("name=a&email=definitely_not_an_email", "missing email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("{}/subscription", &test_app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "the api didn't respond with 400 when the payload was {}",
            error_message
        )
    }
}
