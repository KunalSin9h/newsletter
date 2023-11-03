use actix_web::{get, http::header::ContentType, web, HttpResponse};
use anyhow::Context;
use sqlx::PgPool;

use crate::authentication::UserID;
use crate::utils::e500;

#[get("/dashboard")]
pub async fn admin_dashboard(
    pool: web::Data<PgPool>,
    user_id: web::ReqData<UserID>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user_id.into_inner();

    let username = get_username(*user_id, &pool).await.map_err(e500)?;

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(get_admin_html(username)))
}

#[tracing::instrument(name = "Get username from db", skip(pool))]
pub async fn get_username(user_id: uuid::Uuid, pool: &PgPool) -> Result<String, anyhow::Error> {
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
            <br />
            <p>Available actions:</p>
            <ol>
                <li><a href="/admin/password">Change password</a></li>
                <li>
                    <form name="logoutForm" action="/admin/logout" method="post">
                        <input type="submit" value="Logout">
                    </form>
                </li>
            </ol>
        </body>
    </html>
    "#
    )
}
