use actix_web::http::header::ContentType;
use actix_web::{get, HttpResponse};

#[get("/")]
pub async fn home() -> HttpResponse {
    let home_page = include_str!("index.html");
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(home_page)
}
