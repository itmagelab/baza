use core::pgp;

use clap::{Parser, Subcommand};

mod bundle;
mod password;

use tracing_subscriber::{fmt, EnvFilter};

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    Init,
    Bundle(bundle::Args),
    Password(password::Args),
}

#[derive(Parser, Debug)]
#[command(name = "baza")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
#[tracing::instrument]
pub async fn main() {
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .compact()
        .init();

    let args = Cli::parse();
    let result = match args.command {
        Commands::Password(s) => password::handle(s),
        Commands::Bundle(s) => bundle::handle(s),
        Commands::Init => {
            pgp::generate();
            core::init();
            Ok(())
        }
    };
    match result {
        Ok(_) => (),
        Err(err) => {
            tracing::error!(error = ?err, "{err}");
            std::process::exit(1);
        }
    };
}
