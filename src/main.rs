use newsletter::run;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let lst = TcpListener::bind("127.0.0.1:8000").expect("Failed to bind :8000 at main");
    println!("Started server at post 8000");
    run(lst)?.await
}
