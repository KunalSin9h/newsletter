use std::net::TcpListener;

use newsletter::configuration::get_configuration;
use newsletter::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Panic if we did't read the configuration
    let configuration = get_configuration().expect("Failed to read configuration.");

    // A tcp listener for listening on port
    let lst = TcpListener::bind(format!("127.0.0.1:{}", configuration.application_port))?;

    println!("Started server at post {}", configuration.application_port);
    run(lst)?.await
}
