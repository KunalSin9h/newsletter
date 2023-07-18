use newsletter::configuration;
use newsletter::startup::spawn_app;
use sqlx::{Connection, PgConnection};

#[tokio::test]
async fn subscribe_return_a_200_for_valid_form_data() {
    let address = spawn_app();
    let config = configuration::get_configuration().expect("Failed to read configuration");
    let connection_string = config.database.connection_string();

    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("Failed to connect to postgres");

    let client = reqwest::Client::new();

    let body = "name=Kunal%20Singh&email=kunal%40gmail.com";

    let response = client
        .post(format!("{}/subscribe", &address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name from subscriptions",)
        .fetch_one(&mut connection)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "saved@email.com");
    assert_eq!(saved.name, "Saved Name");
}

#[tokio::test]
async fn subscribe_return_a_400_when_data_is_missing() {
    let address = spawn_app();
    let client = reqwest::Client::new();

    let test_cases = vec![
        ("name=Kunal%20Singh", "missing email in the body"),
        ("email=kunal%40gmail.com", "missing name in the body"),
        ("", "missing both(name & email) in the body"),
    ];

    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(format!("{}/subscribe", address))
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
