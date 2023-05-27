use newsletter::run;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // Get port from environment variable
    // if not specified then use a random port available
    let mut port = std::env::var("SERVER_PORT").unwrap_or("0".into());

    // A tcp listener for listening on port
    let lst = TcpListener::bind(format!("127.0.0.1:{}", port))
        .unwrap_or_else(|_| panic!("Failed to create TCP Listener at port: {}", port));

    // Get random port if any
    port = lst.local_addr().unwrap().port().to_string();

    println!("Started server at post {}", port);
    run(lst)?.await
}
