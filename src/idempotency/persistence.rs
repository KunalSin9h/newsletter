use actix_web::HttpResponse;
use reqwest::StatusCode;
use sqlx::PgPool;

use super::IdempotencyKey;

// This is a representation of a composite type in our database
#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "header_pair")]
struct HeaderPairRecord {
    name: String,
    value: Vec<u8>,
}

pub async fn get_saved_response(
    pool: &PgPool,
    idempotency_key: &IdempotencyKey,
    user_id: &uuid::Uuid,
) -> Result<Option<HttpResponse>, anyhow::Error> {
    let user_id = sqlx::types::Uuid::from_bytes(user_id.into_bytes());

    let saved_response = sqlx::query!(
        r#"
        SELECT 
            response_status_code,
            response_headers as "response_headers: Vec<HeaderPairRecord>",
            response_body
        FROM idempotency
        WHERE
            user_id = $1 AND
            idempotency_key = $2
        "#,
        user_id,
        idempotency_key.as_ref()
    )
    .fetch_optional(pool)
    .await?;

    if let Some(r) = saved_response {
        let status_code = StatusCode::from_u16(r.response_status_code.try_into()?)?;

        let mut response = HttpResponse::build(status_code);

        for HeaderPairRecord { name, value } in r.response_headers {
            response.append_header((name, value));
        }

        Ok(Some(response.body(r.response_body)))
    } else {
        Ok(None)
    }
}
