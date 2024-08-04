use eyre::Context;
use rdkafka::{
    message::OwnedHeaders,
    producer::{FutureProducer, FutureRecord},
};
use sqlx::{types::Json, Connection, PgConnection};
use tokio::time::{sleep, timeout, Duration};

use crate::db_models::{Headers, OutboxMessage};

enum StepStatus {
    QueueIsEmpty,
    MessageSent,
}

pub async fn run_producer(db_dsn: &str, bootstrap_servers: &str) -> eyre::Result<()> {
    // TODO: Add supervisor to restart producer on failure

    let mut db = PgConnection::connect(db_dsn)
        .await
        .wrap_err_with(|| format!("Failed to connect to {}", db_dsn))?;

    let producer: FutureProducer = rdkafka::ClientConfig::new()
        .set("bootstrap.servers", bootstrap_servers)
        .create()
        .wrap_err_with(|| format!("Failed to create producer for {}", bootstrap_servers))?;

    loop {
        match send_next_message(&mut db, &producer).await? {
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

async fn send_next_message(
    db: &mut PgConnection,
    producer: &FutureProducer,
) -> eyre::Result<StepStatus> {
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
    dbg!(&row);

    let mut record = FutureRecord::to(&row.topic);
    if let Some(ref payload) = row.payload {
        record = record.payload(payload);
    }
    if let Some(ref key) = row.key {
        record = record.key(key);
    }
    if let Some(Json(ref headers)) = row.headers {
        let mut owned_headers = OwnedHeaders::new_with_capacity(headers.len());
        for (key, value) in headers.iter() {
            owned_headers = owned_headers.insert(rdkafka::message::Header {
                key,
                value: value.as_ref(),
            });
        }
        record = record.headers(owned_headers);
    }
    let delivery_status = timeout(
        // Short deadline to avoid row locking for long time
        // TODO: Make it configurable
        Duration::from_millis(100),
        producer.send(record, Duration::from_secs(0)),
    )
    .await
    .wrap_err("Failed to send message")?
    .map_err(|(err, _)| eyre::Error::from(err).wrap_err("Failed to send message"))?;
    dbg!(delivery_status);

    tnx.commit().await?;
    Ok(StepStatus::MessageSent)
}
