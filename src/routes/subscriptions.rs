use actix_web::{web, Responder};

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

// POST /subscribe
// Subscribe to email newsletter
pub async fn subscribe(form: web::Form<FormData>) -> impl Responder {
    format!("Name -> {} & Email -> {}", form.name, form.email)
}
