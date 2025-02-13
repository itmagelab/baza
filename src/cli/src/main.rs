use baza_core::{container, error::Error};

use clap::{CommandFactory, Parser, Subcommand};

mod bundle;
mod password;

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
    /// Unlock database
    Unlock,
    /// Lock database
    Lock,
    /// Initializing the database
    Init {
        #[arg(short, long)]
        passphrase: Option<String>,
    },
    /// Work this passwords (bundles)
    Bundle(bundle::Args),
    /// Generating a password
    Password(password::Args),
}

#[derive(Parser, Debug)]
#[command(name = "baza")]
pub struct Cli {
    /// Create bundle of passwords
    #[arg(short, long)]
    create: Option<String>,
    /// Edit exists bundle of passwords
    #[arg(short, long)]
    edit: Option<String>,
    /// Deleting a bundle
    #[arg(short, long)]
    delete: Option<String>,
    /// Search bundle by name
    #[arg(short, long)]
    search: Option<String>,
    /// Copy all bundle to clipboard
    #[arg(long)]
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
    } else if let Some(s) = args.edit {
        container::edit(s)
    } else if let Some(s) = args.delete {
        container::delete(s)
    } else if let Some(s) = args.search {
        container::search(s)
    } else if let Some(s) = args.create {
        container::create(s)
    } else if let Some(command) = args.command {
        match command {
            Commands::Password(s) => password::handle(s),
            Commands::Bundle(s) => bundle::handle(s),
            Commands::Init { passphrase } => baza_core::init(passphrase),
            Commands::Unlock => baza_core::unlock(),
            Commands::Lock => baza_core::lock(),
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
