use crate::{
    configuration::Settings, domain::SubscriberEmail, email_client::EmailClient,
    startup::get_connection_pool,
};
use sqlx::{PgPool, Postgres, Transaction};
use tracing::{field::display, Span};

pub async fn run_worker_until_stopped(configuration: Settings) {
    let pool = get_connection_pool(&configuration.database);
    let email_client = configuration.email_client.client();

    worker_loop(&pool, &email_client)
        .await
        .expect("Failed to start worker loop");
}

async fn worker_loop(pool: &PgPool, email_client: &EmailClient) -> Result<(), anyhow::Error> {
    loop {
        match try_execute_task(pool, email_client).await {
            Ok(ExecutionOutput::EmptyQueue) => {
                tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            }
            Ok(ExecutionOutput::TaskCompleted) => {}
            // TODO: exponential backoff with jitter
            Err(_) => {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
        };
    }
}

pub enum ExecutionOutput {
    TaskCompleted,
    EmptyQueue,
}

#[tracing::instrument(
    skip_all,
    fields(
        newsletter_issue_id=tracing::field::Empty,
        subscriber_email=tracing::field::Empty
    ),
    err
)]
pub async fn try_execute_task(
    pool: &PgPool,
    email_client: &EmailClient,
) -> Result<ExecutionOutput, anyhow::Error> {
    let task = dequeue_task(pool).await?;
    if task.is_none() {
        return Ok(ExecutionOutput::EmptyQueue);
    }

    let (transaction, newsletter_issue_id, subscriber_email) = task.unwrap();

    Span::current()
        .record("newsletter_issue_id", &display(&newsletter_issue_id))
        .record("subscriber_email", &display(&subscriber_email));

    // FUTURE TODO
    // Use cache to store newsletter_issue
    // so that we don't hit the database all the time
    let issue_data = get_issue(pool, newsletter_issue_id).await?;

    match SubscriberEmail::parse(subscriber_email.clone()) {
        Ok(email) => {
            // TODO: Retry
            // We are only trying sending email only once
            // so introduce retry
            if let Err(e) = email_client
                .send_email(
                    &email,
                    &issue_data.title,
                    &issue_data.html,
                    &issue_data.text,
                )
                .await
            {
                tracing::error!(
                    error.cause_chain = ?e,
                    error.message = %e,
                    "Failed to deliver issue to a confirmed subscriber. \
                    Skipping.",
                );
            }
        }
        Err(e) => {
            tracing::error!(
                error.cause_chain = ?e,
                error.message = %e,
                "Skipping a confirmed subscriber. \
                Their stored contact details are invalid",
            );
        }
    }

    delete_task(transaction, newsletter_issue_id, &subscriber_email).await?;

    Ok(ExecutionOutput::TaskCompleted)
}

type PgTransaction = Transaction<'static, Postgres>;

#[tracing::instrument(skip_all)]
async fn dequeue_task(
    pool: &PgPool,
) -> Result<Option<(PgTransaction, sqlx::types::Uuid, String)>, anyhow::Error> {
    let mut transaction = pool.begin().await?;

    let r = sqlx::query!(
        r#"
        SELECT newsletter_issue_id, subscriber_email FROM issue_delivery_queue
        FOR UPDATE
        SKIP LOCKED
        LIMIT 1
    "#
    )
    .fetch_optional(&mut transaction)
    .await?;

    if let Some(row) = r {
        Ok(Some((
            transaction,
            row.newsletter_issue_id,
            row.subscriber_email,
        )))
    } else {
        Ok(None)
    }
}

struct NewsletterIssueData {
    title: String,
    text: String,
    html: String,
}

#[tracing::instrument(skip_all)]
async fn get_issue(
    pool: &PgPool,
    newsletter_issue_id: sqlx::types::Uuid,
) -> Result<NewsletterIssueData, anyhow::Error> {
    let issue_data = sqlx::query_as!(
        NewsletterIssueData,
        r#"
            SELECT title, text, html from newsletter_issue
            WHERE 
                newsletter_issue_id = $1
        "#,
        newsletter_issue_id
    )
    .fetch_one(pool)
    .await?;

    Ok(issue_data)
}

#[tracing::instrument(skip_all)]
async fn delete_task(
    mut trans: PgTransaction,
    newsletter_issue_id: sqlx::types::Uuid,
    email: &str,
) -> Result<(), anyhow::Error> {
    sqlx::query!(
        r#"
        DELETE FROM issue_delivery_queue
        WHERE
            newsletter_issue_id = $1 AND
            subscriber_email = $2
    "#,
        newsletter_issue_id,
        email
    )
    .execute(&mut trans)
    .await?;

    trans.commit().await?;
    Ok(())
}
