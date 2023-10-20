use actix_web::{get, http::header::ContentType, HttpResponse};

#[get("/login")]
pub async fn login_form() -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(include_str!("login.html"))
}
