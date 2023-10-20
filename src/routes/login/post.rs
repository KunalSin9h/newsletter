use actix_web::error::InternalError;
use actix_web::http::header::LOCATION;
use actix_web::http::StatusCode;
use actix_web::{post, web, HttpResponse};
use hmac::{Hmac, Mac};
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;

use crate::authentication::{validate_credential, AuthError, Credentials};
use crate::routes::error_chain_printer;

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>,
}

#[tracing::instrument(skip(form, pool), fields(username=tracing::field::Empty, user_id=tracing::field::Empty))]
#[post("/login")]
pub async fn login(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    secret: web::Data<Secret<String>>,
) -> Result<HttpResponse, InternalError<LoginError>> {
    let credential = Credentials {
        username: form.0.username,
        password: form.0.password,
    };
    tracing::Span::current().record("username", &tracing::field::display(&credential.username));

    match validate_credential(credential, &pool).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));

            Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/"))
                .finish())
        }
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
            };

            let error_message = e.to_string();
            let encoded_error = urlencoding::encode(&error_message);

            let query_string = format!("error={}", encoded_error);

            let secret_key: &[u8] = secret.expose_secret().as_bytes();
            let hmac_tag = {
                let mut mac = Hmac::<sha2::Sha256>::new_from_slice(secret_key).unwrap();
                mac.update(query_string.as_bytes());
                mac.finalize().into_bytes()
            };

            let response = HttpResponse::build(StatusCode::SEE_OTHER)
                .insert_header((
                    LOCATION,
                    format!("/login?{}&tag={hmac_tag:x}", query_string),
                ))
                .finish();

            Err(InternalError::from_response(e, response))
        }
    }
}

#[derive(thiserror::Error)]
pub enum LoginError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    #[error("Something went wrong")]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_printer(self, f)
    }
}
