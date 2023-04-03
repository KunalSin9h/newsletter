use actix_web::{guard, App, HttpResponse, HttpServer, Responder, Route};

async fn health_check() -> impl Responder {
    HttpResponse::Ok()
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    // HttpServer handles all transport level concerns.
    HttpServer::new(|| {
        // App is where all your application logic lives: routing, middlewares, request handlers, etc.
        App::new().route(
            "/health_check",
            Route::new().guard(guard::Get()).to(health_check),
        )
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
