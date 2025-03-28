//! This project is created as an alternative to password-store,
//! but written in a low-level language with additional features
use baza_core::{cleanup_tmp_folder, container, error::Error, sync, Config};

use clap::{CommandFactory, Parser, Subcommand};

mod bundle;
mod password;

use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};

#[derive(Debug, Subcommand)]
#[command(
    about = "Baza: The base password manager",
    override_usage = r#"baza [-a <bundle> | --add <bundle>] [-d <bundle> | --delete <bundle>]
            [-e <bundle> | --edit <bundle>] [-c <bundle> | --copy <bundle>]
            [-s <bundle> | --search <bundle>] [-p <bundle> | --show <bundle>]
            [-v | --version] [-h | --help] [-l | --list]
            [<command>] [<args>]

    "#,
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
    /// Load database
    Sync,
    /// Work with passwords (bundles)
    Bundle(bundle::Args),
    /// Generating a password
    Password(password::Args),
}

#[derive(Parser, Debug)]
#[command(name = "baza")]
pub struct Cli {
    /// Adding bundle of passwords
    ///
    /// baza [--add my::secret::login | -a my::secret::login]
    #[arg(short, long, value_name = "BUNDLE")]
    add: Option<String>,
    /// Create bundle from STDIN
    #[arg(long, value_name = "BUNDLE")]
    stdin: Option<String>,
    /// Edit exists bundle of passwords
    #[arg(short, long, value_name = "BUNDLE")]
    edit: Option<String>,
    /// Deleting a bundle
    #[arg(short, long, value_name = "BUNDLE")]
    delete: Option<String>,
    /// Search bundle by name
    #[arg(short, long, value_name = "REGEX")]
    search: Option<String>,
    /// Copy all bundle to clipboard
    #[arg(short, long, value_name = "BUNDLE")]
    copy: Option<String>,
    /// Show Version
    #[arg(short, long)]
    version: bool,
    #[command(subcommand)]
    command: Option<Commands>,
    /// Show content of bundle
    #[arg(short = 'p', long, value_name = "BUNDLE")]
    show: Option<String>,
    /// List all containers
    #[arg(short, long)]
    list: bool,
}

#[tokio::main]
pub async fn main() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    cleanup_tmp_folder().unwrap();

    let fmt = fmt::layer()
        .with_target(false)
        .without_time()
        .compact()
        .json()
        .with_filter(filter);
    tracing_subscriber::registry().with(fmt).init();

    tracing::debug!(datadir = &Config::get().main.datadir, "Use datadir");

    let args = Cli::parse();
    let result = if let Some(s) = args.copy {
        container::copy_to_clipboard(s)
    } else if let Some(s) = args.show {
        container::read(s)
    } else if let Some(s) = args.edit {
        container::update(s)
    } else if let Some(s) = args.delete {
        container::delete(s)
    } else if let Some(s) = args.search {
        container::search(s)
    } else if let Some(s) = args.add {
        container::create(s)
    } else if let Some(s) = args.stdin {
        container::from_stdin(s)
    } else if args.list {
        container::search(String::from(".*"))
    } else if args.version {
        println!("{}", env!("CARGO_PKG_VERSION"));
        Ok(())
    } else if let Some(command) = args.command {
        match command {
            Commands::Password(s) => password::handle(s),
            Commands::Bundle(s) => bundle::handle(s),
            Commands::Init { passphrase } => baza_core::init(passphrase),
            Commands::Unlock => baza_core::unlock(None),
            Commands::Lock => baza_core::lock(),
            Commands::Sync => sync(),
        }
    } else {
        Cli::command().print_long_help().map_err(Error::IO)
    };
    match result {
        Ok(_) => (),
        Err(err) => {
            tracing::error!(error = ?err, "{err}");
            std::process::exit(1);
        }
    };
}
