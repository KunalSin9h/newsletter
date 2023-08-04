use std::net::TcpListener;

use newsletter::configuration::get_configuration;
use newsletter::startup::run;
use newsletter::telemetry::{get_subscriber, init_subscriber};
use secrecy::ExposeSecret;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("newsletter".into(), "info".into());
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");

    let connection_pool =
        PgPool::connect(&configuration.database.connection_string().expose_secret())
            .await
            .expect("Failed to connect to Postgres database");

    // A tcp listener for listening on port
    let lst = TcpListener::bind(format!("127.0.0.1:{}", configuration.application_port))?;

    println!("Started server at post {}", configuration.application_port);
    run(lst, connection_pool)?.await
}
