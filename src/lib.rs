use actix_web::dev::Server;
use actix_web::{guard, App, HttpResponse, HttpServer, Route};
use std::net::TcpListener;

async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new().route(
            "/health_check",
            Route::new().guard(guard::Get()).to(health_check),
        )
    })
    .listen(listener)?
    .run();

    Ok(server)
}
