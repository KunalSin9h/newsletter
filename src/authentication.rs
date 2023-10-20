use crate::telemetry::spawn_blocking_task_with_tracing;
use anyhow::Context;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use secrecy::{ExposeSecret, Secret};
use sqlx::{types::Uuid, PgPool};

#[derive(thiserror::Error, Debug)]
pub enum AuthError {
    #[error("Invalid Credentials.")]
    InvalidCredentials(#[source] anyhow::Error),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

pub struct Credentials {
    pub username: String,
    pub password: Secret<String>,
}

#[tracing::instrument(name = "Validate Credentials", skip(credential, pool))]
pub async fn validate_credential(
    credential: Credentials,
    pool: &PgPool,
) -> Result<sqlx::types::Uuid, AuthError> {
    // Prevention from TIMEING ATTACK
    //
    // When username is valid but password is not valid the response time
    // is significantly large then then both password and username is Invalid
    // this difference in response time can be exploited to determine if username is
    // present in the database, which in turn can be used for more sophisticated attach
    // on that person.
    //
    // TO DEAL WITH IT
    //
    // We are going to let the flow of program to the end even if the username is invalid
    // in the first place
    // __________________________________________________________________________________
    let mut user_id: Option<Uuid> = None;
    // some random password hash
    let mut expected_password_hash = Secret::new(
        "$argon2id$v=19$m=15000,t=2,p=1$\
                gZiV/M1gPc22ElAH/Jh1Hw$\
                CWOrkoo7oJBQ/iyh7uJ0LO2aLEfrHwTWllSAxT0zRno"
            .to_string(),
    );

    if let Some((stored_user_id, stored_hashed_password)) =
        get_stored_credentials(&credential.username, pool)
            .await
            .map_err(AuthError::UnexpectedError)?
    {
        user_id = Some(stored_user_id);
        expected_password_hash = stored_hashed_password;
    }

    spawn_blocking_task_with_tracing(move || {
        verify_password_hash(expected_password_hash, credential.password)
    })
    .await
    .context("Failed to spawn blocking task.")
    .map_err(AuthError::UnexpectedError)?
    .await?;

    // .map_err(AuthError::UnexpectedError)?
    // .await?;
    // .map_err(AuthError::InvalidCredentials)?
    // .context("Failed to spawn blocking task.")
    // .await?;

    user_id.ok_or_else(|| AuthError::InvalidCredentials(anyhow::anyhow!("Unknown Username.")))
}

#[tracing::instrument(
    name = "Verify password hash",
    skip(expected_password_hash, original_password)
)]
pub async fn verify_password_hash(
    expected_password_hash: Secret<String>,
    original_password: Secret<String>,
) -> Result<(), AuthError> {
    // we are parsing that PHC Password string
    // converting row phc string to PasswordHash Struct
    let expected_password = PasswordHash::new(expected_password_hash.expose_secret())
        .context("Failed to parse hash in PHC string format")?;

    Argon2::default()
        .verify_password(
            original_password.expose_secret().as_bytes(),
            &expected_password,
        )
        .context("Invalid password")
        .map_err(AuthError::InvalidCredentials)
}

#[tracing::instrument(name = "Get Stored Credentials", skip(username, pool))]
pub async fn get_stored_credentials(
    username: &str,
    pool: &PgPool,
) -> Result<Option<(sqlx::types::Uuid, Secret<String>)>, anyhow::Error> {
    let row: Option<_> = sqlx::query!(
        r#"
        SELECT user_id, password_hash
        FROM users
        WHERE username = $1
    "#,
        username
    )
    .fetch_optional(pool)
    .await
    .context("Failed to execute query to validate user credential")?
    .map(|row| (row.user_id, Secret::new(row.password_hash)));

    Ok(row)
}
