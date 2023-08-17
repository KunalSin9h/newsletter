use std::net::TcpListener;

use newsletter::configuration::get_configuration;
use newsletter::email_client::EmailClient;
use newsletter::startup::run;
use newsletter::telemetry::{get_subscriber, init_subscriber};
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("newsletter".into(), "info".into());
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");

    let connection_pool = PgPoolOptions::new()
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());

    let sender = configuration
        .email_client
        .sender()
        .expect("Invalid sender email");

    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender,
        configuration.email_client.authorization_token,
        timeout,
    )
    .expect("Failed to parse email server url");

    // A tcp listener for listening on port
    let lst = TcpListener::bind(format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    ))?;

    println!("Started server at post {}", configuration.application.port);
    run(lst, connection_pool, email_client)?.await
}
