use clap::{Parser, Subcommand};

use bus_outbox::{run_producer, upgrade_db};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long, env)]
    db_dsn: String,
    // TODO: Add logging options
    #[command(subcommand)]
    cmd: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run migration to upgrade database
    Migrate,
    /// Run producer daemon
    Produce {
        #[arg(long, env)]
        bootstrap_servers: String,
        // TODO: Arbitrary options (key-value pairs) to pass directly to `ClientConfig`
    },
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    let args = Args::parse();

    tokio::select! {
        res = run_command(args) => {res?},
        // XXX: This way no clean-up is run
        res = tokio::signal::ctrl_c() => {
            res?;
            // TODO: Log message
        },
    }

    Ok(())
}

async fn run_command(args: Args) -> eyre::Result<()> {
    match args.cmd {
        Commands::Migrate => upgrade_db(&args.db_dsn).await,
        Commands::Produce { bootstrap_servers } => {
            run_producer(&args.db_dsn, &bootstrap_servers).await
        }
    }
}
