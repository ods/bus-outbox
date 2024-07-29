use eyre::Context;
use sqlx::{types::Json, Connection, PgConnection};
use tokio::time::{sleep, Duration};

use crate::db_models::{Headers, OutboxMessage};

enum StepStatus {
    QueueIsEmpty,
    MessageSent,
}

pub async fn run_producer(db_dsn: &str, _bootstrap_servers: &str) -> eyre::Result<()> {
    // TODO: Add supervisor to restart producer on failure

    let mut db = PgConnection::connect(db_dsn)
        .await
        .wrap_err_with(|| format!("Failed to connect to {}", db_dsn))?;

    loop {
        match send_next_message(&mut db).await? {
            StepStatus::QueueIsEmpty => {
                // TODO: Configurable poll interval
                sleep(Duration::from_secs(1)).await;
            }
            StepStatus::MessageSent => {
                // FIXME: temporary delay to avoid busy loop
                sleep(Duration::from_secs(1)).await;
            }
        }
    }
}

async fn send_next_message(db: &mut PgConnection) -> eyre::Result<StepStatus> {
    let mut tnx = db.begin().await?;

    let maybe_row = sqlx::query_as!(
        OutboxMessage,
        r#"
        SELECT id, topic, payload, key, headers as "headers: Json<Headers>"
        FROM bus_outbox_messages
        ORDER BY id
        LIMIT 1
        FOR UPDATE
        "#
    )
    .fetch_optional(&mut *tnx)
    .await?;
    let Some(row) = maybe_row else {
        return Ok(StepStatus::QueueIsEmpty);
    };
    dbg!(row);

    tnx.commit().await?;
    Ok(StepStatus::MessageSent)
}
