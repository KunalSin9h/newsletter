use std::net::TcpListener;

use newsletter::configuration::get_configuration;
use newsletter::startup::run;
use sqlx::PgPool;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configuration().expect("Failed to read configuration.");

    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres database");

    // A tcp listener for listening on port
    let lst = TcpListener::bind(format!("127.0.0.1:{}", configuration.application_port))?;

    println!("Started server at post {}", configuration.application_port);
    run(lst, connection_pool)?.await
}
