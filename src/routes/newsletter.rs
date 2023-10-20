use super::error_chain_printer;
use crate::authentication::{validate_credential, AuthError, Credentials};
use crate::{domain::SubscriberEmail, email_client::EmailClient};
use actix_web::http::{header, StatusCode};
use actix_web::{
    http::header::{HeaderMap, HeaderValue},
    web, HttpRequest, HttpResponse,
};
use anyhow::Context;
use base64::{engine::general_purpose, Engine as _};
use secrecy::Secret;
use sqlx::PgPool;

#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    text: String,
    html: String,
}

pub struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[tracing::instrument(
    name = "Publish a newsletter issue",
    skip(body, pool, email_client, request),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn publish_newsletter(
    body: web::Json<BodyData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    request: HttpRequest,
) -> Result<HttpResponse, PublishError> {
    let credential = basic_authentication(request.headers()).map_err(PublishError::AuthError)?;
    tracing::Span::current().record("username", tracing::field::display(&credential.username));

    let user_id = validate_credential(credential, &pool)
        .await
        .map_err(|e| match e {
            AuthError::UnexpectedError(_) => PublishError::UnexpectedError(e.into()),
            AuthError::InvalidCredentials(_) => PublishError::AuthError(e.into()),
        })?;

    tracing::Span::current().record("user_id", tracing::field::display(&user_id));

    let subscribers = get_confirmed_subscribers(&pool).await?;
    for subscriber in subscribers {
        match subscriber {
            Ok(confirmed_subscriber) => {
                email_client
                    .send_email(
                        &confirmed_subscriber.email,
                        &body.title,
                        &body.content.html,
                        &body.content.text,
                    )
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to send newsletter issue to {}",
                            confirmed_subscriber.email
                        )
                    })?;
            }

            Err(error) => {
                tracing::warn!(error.cause_chain = ?error)
            }
        }
    }

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
pub async fn get_confirmed_subscribers(
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

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error("Authentication error")]
    AuthError(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_printer(self, f)
    }
}

impl actix_web::ResponseError for PublishError {
    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        match self {
            PublishError::UnexpectedError(_) => {
                HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
            }
            PublishError::AuthError(_) => {
                let mut response = HttpResponse::new(StatusCode::UNAUTHORIZED);
                let header_value = HeaderValue::from_str(r#"Basic realm="publish""#).unwrap();

                response
                    .headers_mut()
                    .insert(header::WWW_AUTHENTICATE, header_value);

                response
            }
        }
    }
}

fn basic_authentication(headers: &HeaderMap) -> Result<Credentials, anyhow::Error> {
    let header_value = headers
        .get("Authorization")
        .context(r#"The "Authorization" header is missing"#)?
        .to_str()
        .context(r#"The "Authorization" header is not a valid UTF-8 string"#)?;

    let base64encoded_segment = header_value
        .strip_prefix("Basic ")
        .context("The authorization scheme is not basic")?;

    let decoded_bytes = general_purpose::STANDARD
        .decode(base64encoded_segment)
        .context("Failed to base64-decode the basic credential")?;

    let decoded_credentials = String::from_utf8(decoded_bytes)
        .context("The decoded credential string is not a valid UTF-8 sequence")?;

    let mut credentials = decoded_credentials.splitn(2, ':');

    let username = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!(r#"A username must be provided in the "Basic" auth"#))?
        .to_string();

    let password = credentials
        .next()
        .ok_or_else(|| anyhow::anyhow!(r#"A password must be provided in the "Basic" auth"#))?
        .to_string();

    Ok(Credentials {
        username,
        password: Secret::new(password),
    })
}
