use newsletter::configuration::{get_configuration, DatabaseSettings};
use newsletter::startup::Application;
use newsletter::telemetry::{get_subscriber, init_subscriber};
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::sync::Once;

use uuid::Uuid;

static START: Once = Once::new();

use wiremock::MockServer;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
    pub email_server: MockServer,
}

impl TestApp {
    pub async fn post_subscriptions(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscription", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

pub async fn spawn_app() -> TestApp {
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber("test".into(), "debug".into());
        START.call_once(|| {
            init_subscriber(subscriber);
        });
    }

    let email_server = MockServer::start().await;

    let configuration = {
        let mut c = get_configuration().expect("Failed to get configuration");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c.email_client.base_url = email_server.uri();
        c
    };

    let pg_pool = configure_database(&configuration.database).await;

    let app = Application::build(configuration.clone())
        .await
        .expect("Failed to create application");

    let address = format!("http://127.0.0.1:{}", app.port());
    let _ = tokio::spawn(app.run_until_stopped());

    TestApp {
        address,
        db_pool: pg_pool,
        email_server,
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create the database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres Database");

    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, &config.database_name).as_str())
        .await
        .expect("Failed to create database");

    // Migrate Database
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to create connection pool");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
