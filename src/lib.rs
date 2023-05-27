use actix_web::dev::Server;
use actix_web::{web, App, HttpResponse, HttpServer};
use std::net::TcpListener;

// GET /health_check
// Health Check is an basic endpoint for check the status of the server
async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

// POST /subscribe
// Subscribe to email newsletter
async fn subscribe() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .route("/health_check", web::get().to(health_check))
            .route("/subscribe", web::post().to(subscribe))
    })
    .listen(listener)?
    .run();

    return Ok(server);
}
