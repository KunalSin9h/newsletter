use crate::authentication::UserID;
use crate::idempotency::{get_saved_response, save_response, IdempotencyKey};
use crate::utils::{e400, e500, see_other};
use crate::{domain::SubscriberEmail, email_client::EmailClient};
use actix_web::{post, web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use sqlx::PgPool;

#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    text: String,
    html: String,
    idempotency_key: String,
}

pub struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[tracing::instrument(
    name = "Publish a newsletter issue",
    skip(body, pool, email_client),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
#[post("/newsletters")]
pub async fn newsletter_issue(
    body: web::Form<BodyData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    user_id: web::ReqData<UserID>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user_id.into_inner();
    tracing::Span::current().record("user_id", tracing::field::display(*user_id));

    let BodyData {
        title,
        text,
        html,
        idempotency_key,
    } = body.0;
    let idempotency_key: IdempotencyKey = idempotency_key.try_into().map_err(e400)?;

    if let Some(saved_response) = get_saved_response(&pool, &idempotency_key, *user_id)
        .await
        .map_err(e500)?
    {
        return Ok(saved_response);
    }

    let subscribers = get_confirmed_subscribers(&pool).await.map_err(e500)?;
    for subscriber in subscribers {
        match subscriber {
            Ok(confirmed_subscriber) => {
                email_client
                    .send_email(&confirmed_subscriber.email, &title, &html, &text)
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to send newsletter issue to {}",
                            confirmed_subscriber.email
                        )
                    })
                    .map_err(e500)?;
            }

            Err(error) => {
                FlashMessage::error(&error.to_string()).send();
                tracing::warn!(error.cause_chain = ?error)
            }
        }
    }

    FlashMessage::info("The newsletter issue has been published!").send();
    let response = save_response(
        &pool,
        &idempotency_key,
        *user_id,
        see_other("/admin/newsletters"),
    )
    .await
    .map_err(e500)?;
    Ok(response)
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &PgPool,
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    let rows = sqlx::query!(
        r#"
            SELECT email
            FROM subscriptions 
            WHERE status = 'confirmed' 
        "#,
    )
    .fetch_all(pool)
    .await?;

    let confirmed = rows
        .into_iter()
        .map(|r| match SubscriberEmail::parse(r.email) {
            Ok(email) => Ok(ConfirmedSubscriber { email }),
            Err(error) => Err(anyhow::anyhow!(error)),
        })
        .collect();

    Ok(confirmed)
}
