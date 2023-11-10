use chrono::Utc;
use sqlx::PgPool;

use crate::{configuration::Settings, startup::get_connection_pool};

pub async fn run_idempotency_worker(configuration: Settings) {
    let pool = get_connection_pool(&configuration.database);

    loop {
        idempotency_worker(&pool)
            .await
            .expect("Failed to run idempotency_worker");
        tokio::time::sleep(std::time::Duration::from_secs(60 * 60)).await;
    }
}

// in one go it will delete all the rows with created_at entry
// created more then 12 hrs ago
async fn idempotency_worker(pool: &PgPool) -> Result<(), sqlx::Error> {
    let mut transaction = pool.begin().await?;

    let r = sqlx::query!(
        r#"
        SELECT user_id, idempotency_key, created_at
        FROM idempotency
    "#
    )
    .fetch_all(&mut transaction)
    .await?;

    for row in r {
        let now = Utc::now();
        let created_at = row.created_at;
        let diff = now - created_at;

        if diff.num_hours() >= 12 {
            sqlx::query!(
                r#"
                DELETE FROM idempotency
                WHERE user_id = $1 AND idempotency_key = $2
            "#,
                row.user_id,
                row.idempotency_key
            )
            .execute(&mut transaction)
            .await?;
        }
    }

    transaction.commit().await?;

    Ok(())
}
