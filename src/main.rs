use clap::{Parser, Subcommand};

use bus_outbox::migrate::upgrade_db;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, env)]
    db_dsn: String,

    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run migration to upgrade database
    Migrate,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let args = Args::parse();

    match args.cmd {
        Commands::Migrate => upgrade_db(&args.db_dsn).await?,
    }

    Ok(())
}
