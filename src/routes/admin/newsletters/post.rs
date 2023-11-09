use crate::authentication::UserID;
use crate::idempotency::{save_response, try_processing, IdempotencyKey, NextAction};
use crate::utils::{e400, e500, see_other};
use actix_web::{post, web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use sqlx::{PgPool, Postgres, Transaction};

#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    text: String,
    html: String,
    idempotency_key: String,
}

#[tracing::instrument(
    name = "Publish a newsletter issue",
    skip(body, pool),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
#[post("/newsletters")]
pub async fn newsletter_issue(
    body: web::Form<BodyData>,
    pool: web::Data<PgPool>,
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

    let mut transaction = match try_processing(&pool, &idempotency_key, *user_id)
        .await
        .map_err(e500)?
    {
        NextAction::ReturnSaveResponse(resp) => {
            success_message().send();
            return Ok(resp);
        }
        NextAction::StartProcessing(t) => t,
    };

    let newsletter_issue_id = insert_newsletter_issue(&mut transaction, &title, &text, &html)
        .await
        .context("Failed to store newsletter issue details")
        .map_err(e500)?;

    enqueue_delivery_task(&mut transaction, newsletter_issue_id)
        .await
        .context("failed to enqueue delivery details")
        .map_err(e500)?;

    success_message().send();
    let response = save_response(
        transaction,
        &idempotency_key,
        *user_id,
        see_other("/admin/newsletters"),
    )
    .await
    .map_err(e500)?;
    Ok(response)
}

fn success_message() -> FlashMessage {
    FlashMessage::info("The newsletter issue has been published!")
}

#[tracing::instrument(skip_all)]
async fn enqueue_delivery_task(
    transaction: &mut Transaction<'_, Postgres>,
    newsletter_issue_id: uuid::Uuid,
) -> Result<(), sqlx::Error> {
    let sqlx_uuid = sqlx::types::Uuid::from_bytes(newsletter_issue_id.into_bytes());

    sqlx::query!(
        r#"
        INSERT INTO issue_delivery_queue (newsletter_issue_id, subscriber_email)
        SELECT $1, email
        FROM subscriptions
        WHERE status = 'confirmed'
    "#,
        sqlx_uuid
    )
    .execute(transaction)
    .await?;

    Ok(())
}

#[tracing::instrument(skip_all)]
async fn insert_newsletter_issue(
    transaction: &mut Transaction<'_, Postgres>,
    title: &str,
    text: &str,
    html: &str,
) -> Result<uuid::Uuid, sqlx::Error> {
    let newsletter_issue_id = uuid::Uuid::new_v4();
    let sqlx_issue_id = sqlx::types::Uuid::from_bytes(newsletter_issue_id.into_bytes());

    sqlx::query!(
        r#"
        INSERT INTO newsletter_issue (newsletter_issue_id, title, text, html, published_at)
        VALUES ($1, $2, $3, $4, now());
    "#,
        sqlx_issue_id,
        title,
        text,
        html
    )
    .execute(transaction)
    .await?;

    Ok(newsletter_issue_id)
}
