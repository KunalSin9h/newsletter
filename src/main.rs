use std::net::TcpListener;

use newsletter::startup::run;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Get port from environment variable
    // if not specified then use a random port available
    let mut port = std::env::var("SERVER_PORT").unwrap_or("6969".into());

    // A tcp listener for listening on port
    let lst = TcpListener::bind(format!("127.0.0.1:{}", port))
        .unwrap_or_else(|_| panic!("Failed to create TCP Listener at port: {}", port));

    // Get random port if any
    port = lst.local_addr()?.port().to_string();

    println!("Started server at post {}", port);
    run(lst)?.await
}
