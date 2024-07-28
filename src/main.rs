use clap::Parser;
use eyre::Context;
use sqlx::{Connection, PgConnection};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, env)]
    db_dsn: String,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let args = Args::parse();

    let mut db = PgConnection::connect(&args.db_dsn)
        .await
        .wrap_err_with(|| format!("Failed to connect to {}", &args.db_dsn))?;
    sqlx::migrate!().run(&mut db).await?;

    Ok(())
}
