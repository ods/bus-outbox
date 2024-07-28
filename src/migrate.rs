use eyre::Context;
use sqlx::{Connection, PgConnection};

pub async fn upgrade_db(db_dsn: &str) -> eyre::Result<()> {
    let mut db = PgConnection::connect(db_dsn)
        .await
        .wrap_err_with(|| format!("Failed to connect to {}", db_dsn))?;

    sqlx::migrate!().run(&mut db).await?;

    Ok(())
}
