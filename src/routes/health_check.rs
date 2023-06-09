use actix_web::HttpResponse;

// GET /health_check
// Health Check is an basic endpoint for check the status of the server
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().finish()
}
