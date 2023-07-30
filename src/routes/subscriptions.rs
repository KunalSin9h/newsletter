use actix_web::{web, HttpResponse, Responder};
use chrono::Utc;
use sqlx::PgPool;
use tracing::Instrument;
use uuid::Uuid; // For Futures + Tracing

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

// POST /subscribe
// Subscribe to email newsletter
pub async fn subscribe(form: web::Form<FormData>, db_pool: web::Data<PgPool>) -> impl Responder {
    let request_id = Uuid::new_v4();

    let request_span = tracing::info_span!(
        "Adding a new subscriber.",
        %request_id,
        subscribe_email = %form.email,
        subscribe_name = %form.name,
    );

    let _request_span_guard = request_span.enter();

    tracing::info!(
        "request_id {} - Adding '{}' '{}' as a new subscriber",
        request_id,
        form.name,
        form.email
    );
    tracing::info!(
        "request_id {} - Saving new subscriber into the database",
        request_id
    );

    let query_span = tracing::info_span!("Saving new subscribe details into database");

    match sqlx::query!(
        "INSERT INTO subscriptions (id, email, name, subscription_at) VALUES ($1, $2, $3, $4)",
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(db_pool.get_ref())
    .instrument(query_span)
    .await
    {
        Ok(_) => {
            tracing::info!(
                "request_id {} - New subscriber details have been saved",
                request_id
            );
            HttpResponse::Ok().finish()
        }
        Err(error) => {
            tracing::error!(
                "request_id {} - Failed to execute the query: {:?}",
                request_id,
                error
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}

/*
tracing-subscriber
    |
    |-> Subscribers
    |-> Layer
 */
