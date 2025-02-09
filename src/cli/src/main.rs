use core::{container, error::Error};

use clap::{CommandFactory, Parser, Subcommand};

mod bundle;
mod password;
mod storage;

use tracing_subscriber::{fmt, EnvFilter};

#[derive(Debug, Subcommand)]
#[command(
    about = "Baza: The base password manager",
    long_about = r#"
        +-+-+-+-+
        |B|A|Z|A|
        +-+-+-+-+

The base password manager
"#
)]
pub(crate) enum Commands {
    Init {
        #[arg(short, long)]
        uuid: Option<String>,
    },
    Bundle(bundle::Args),
    Password(password::Args),
    Storage(storage::Args),
}

#[derive(Parser, Debug)]
#[command(name = "baza")]
pub struct Cli {
    #[arg(short, long)]
    copy: Option<String>,
    #[command(subcommand)]
    command: Option<Commands>,
}

#[tokio::main]
#[tracing::instrument]
pub async fn main() {
    fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .compact()
        .init();

    let args = Cli::parse();
    let result = if let Some(s) = args.copy {
        container::copy_to_clipboard(s)
    } else if let Some(command) = args.command {
        match command {
            Commands::Password(s) => password::handle(s),
            Commands::Bundle(s) => bundle::handle(s),
            Commands::Init { uuid } => core::init(uuid),
            Commands::Storage(s) => storage::handle(s),
        }
    } else {
        Cli::command().print_long_help().map_err(Error::HelpError)
    };
    match result {
        Ok(_) => (),
        Err(err) => {
            tracing::error!(error = ?err, "{err}");
            std::process::exit(1);
        }
    };
}
