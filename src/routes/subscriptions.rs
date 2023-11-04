use actix_web::{web, HttpResponse};
use chrono::Utc;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use sqlx::types::Uuid;
use sqlx::PgPool;

use crate::{domain::NewSubscriber, email_client::EmailClient};

use super::SubscribeError;

#[derive(serde::Deserialize)]
pub struct FormData {
    name: String,
    email: String,
}

impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(form: FormData) -> Result<Self, Self::Error> {
        let name = form.name.try_into()?;
        let email = form.email.try_into()?;
        Ok(NewSubscriber { name, email })
    }
}

// POST /subscription
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
    base_url: web::Data<String>,
    app_port: web::Data<u16>,
) -> Result<HttpResponse, SubscribeError> {
    let new_subscriber = form.0.try_into()?;

    let mut transition = db_pool.begin().await.map_err(SubscribeError::PoolError)?;

    let subscriber_id = insert_subscriber(&mut transition, &new_subscriber)
        .await
        .map_err(SubscribeError::InsertSubscriberError)?;

    let subscription_token = generate_subscription_token();
    store_token(&mut transition, subscriber_id, &subscription_token).await?;

    transition
        .commit()
        .await
        .map_err(SubscribeError::TransactionCommitError)?;

    send_confirmation_email(
        &email_client,
        new_subscriber,
        &base_url,
        &app_port,
        &subscription_token,
    )
    .await?;

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
    name = "Store the subscription token in the database",
    skip(transition, subscriber_id, subscription_token)
)]
pub async fn store_token(
    transition: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    subscriber_id: Uuid,
    subscription_token: &String,
) -> Result<(), StoreTokenError> {
    sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id) VALUES ($1, $2)"#,
        subscription_token,
        subscriber_id
    )
    .execute(transition)
    .await
    .map_err(|e| {
        tracing::error!("failed to execute query: {:?}", e);
        StoreTokenError(e)
    })?;

    Ok(())
}

pub struct StoreTokenError(sqlx::Error);

pub fn error_chain_printer(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current_source = e.source();

    while let Some(cause) = current_source {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current_source = cause.source();
    }

    Ok(())
}

impl std::fmt::Debug for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_printer(self, f)
    }
}

impl std::fmt::Display for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A database error was encountered while \
            trying to store a subscription token"
        )
    }
}

impl std::error::Error for StoreTokenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

impl actix_web::ResponseError for StoreTokenError {}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_sub, base_url, token)
)]
pub async fn send_confirmation_email(
    email_client: &web::Data<EmailClient>,
    new_sub: NewSubscriber,
    base_url: &web::Data<String>,
    app_port: &web::Data<u16>,
    token: &str,
) -> Result<(), reqwest::Error> {
    let base_host_url = if base_url.contains("127.0.0.1") || base_url.contains("localhost") {
        format!("{}:{}", base_url.as_str(), app_port.as_ref())
    } else {
        base_url.to_string()
    };

    let confirmation_link = format!(
        "{}/subscription/confirm?subscription_token={}",
        base_host_url, token
    );
    let text_body = &format!(
        "Welcome to my Newsletter!\nVisit {} to confirm your subscription",
        confirmation_link
    );

    let html_body = &format!(
        "Welcome to my Newsletter!<br />\
            Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );

    email_client
        .send_email(&new_sub.email, "Welcome", html_body, text_body)
        .await
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, transition)
)]
pub async fn insert_subscriber(
    transition: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    new_subscriber: &NewSubscriber,
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();

    sqlx::query!(
        r#"
    INSERT INTO subscriptions (id, email, name, subscription_at, status)
    VALUES ($1, $2, $3, $4, 'pending_confirmation')
    "#,
        subscriber_id,
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(transition)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(subscriber_id)
}

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();

    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}
