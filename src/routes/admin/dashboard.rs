use actix_web::{get, http::header::ContentType, web, HttpResponse};
use anyhow::Context;
use reqwest::header::LOCATION;
use sqlx::PgPool;

use crate::session_state::TypedSession;
use crate::utils::e500;

#[get("/admin/dashboard")]
pub async fn admin_dashboard(
    session: TypedSession,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let username = if let Some(user_id) = session.get_user_id().map_err(e500)? {
        get_username(user_id, &pool).await.map_err(e500)?
    } else {
        return Ok(HttpResponse::SeeOther()
            .insert_header((LOCATION, "/login"))
            .finish());
    };

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(get_admin_html(username)))
}

#[tracing::instrument(name = "Get username from db", skip(pool))]
async fn get_username(user_id: uuid::Uuid, pool: &PgPool) -> Result<String, anyhow::Error> {
    let row = sqlx::query!(
        r#"
            SELECT username
            FROM users
            WHERE user_id = $1
        "#,
        sqlx::types::Uuid::from_bytes(user_id.into_bytes())
    )
    .fetch_one(pool)
    .await
    .context("failed to perform query to retrieve a username")?;

    Ok(row.username)
}

fn get_admin_html(username: String) -> String {
    format!(
        r#"
    <html lang="en">
        <head>
            <meta http-equiv="content-type" content="text/html; charset=utf-8">
            <title>Admin dashboard</title>
        </head>
        <body>
            <p>Welcome {username}!</p>
        </body>
    </html>
    "#
    )
}
