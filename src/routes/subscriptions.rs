use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;

use crate::{domain::NewSubscriber, email_client::EmailClient};

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

pub fn parse_subscriber(form: FormData) -> Result<NewSubscriber, String> {
    let name = form.name.try_into()?;
    let email = form.email.try_into()?;
    Ok(NewSubscriber { name, email })
}

// POST /subscribe
// Subscribe to email newsletter
#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, db_pool, email_client),
    fields(
        subscriber_email = %form.email,
        subscriber_name= %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    db_pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> HttpResponse {
    let new_subscriber = match parse_subscriber(form.0) {
        Ok(subscriber) => subscriber,
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    if insert_subscriber(&db_pool, &new_subscriber).await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    let confirmation_link =
        "http://newsletter.kunalsin9h.com/subscription/confirm?token=random_token";

    if email_client
        .send_email(
            new_subscriber.email,
            "Welcome",
            &format!(
                "Welcome to my Newsletter!<br />\
            Click <a href=\"{}\">here</a> to confirm your subscription.",
                confirmation_link
            ),
            &format!(
                "Welcome to my Newsletter!\nVisit {} to confirm your subscription",
                confirmation_link
            ),
        )
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscription_at, status)
    VALUES ($1, $2, $3, $4, 'confirmed')
    "#,
        sqlx::types::Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
