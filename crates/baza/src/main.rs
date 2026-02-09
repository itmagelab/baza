use baza_core::{cleanup_tmp_folder, container, sync, BazaR};

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
    /// List all containers
    List,
    /// Show Version
    Version,
}

impl Commands {
    fn run(self) -> BazaR<()> {
        match self {
            Commands::Password(s) => password::handle(s)?,
            Commands::Bundle(s) => bundle::handle(s)?,
            Commands::Init { passphrase } => baza_core::init(passphrase)?,
            Commands::Unlock => baza_core::unlock(None)?,
            Commands::Lock => baza_core::lock()?,
            Commands::Sync => sync()?,
            Commands::List => {
                container::search(String::from(".*"))?;
            }
            Commands::Version => {
                println!("{}", env!("CARGO_PKG_VERSION"));
            }
        };
        Ok(())
    }
}

#[derive(Parser, Debug)]
#[command(name = "baza", version)]
pub struct Cli {
    /// Adding bundle of passwords
    #[arg(short, long, value_name = "BUNDLE")]
    add: Option<String>,

    /// Create bundle from STDIN
    #[arg(long, value_name = "BUNDLE")]
    stdin: Option<String>,

    /// Generate default bundle
    #[arg(short, long, value_name = "BUNDLE", num_args = 0..=1)]
    generate: Option<String>,

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

    /// Show content of bundle
    #[arg(short = 'p', long, value_name = "BUNDLE")]
    show: Option<String>,

    /// List all containers
    #[arg(short, long)]
    list: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

impl Cli {
    /// Maps top-level shortcut flags to their corresponding subcommands
    fn get_command(self) -> Option<Commands> {
        if self.list {
            return Some(Commands::List);
        }

        if let Some(name) = self.copy {
            return Some(Commands::Bundle(bundle::Args {
                command: bundle::Commands::Copy { name },
            }));
        }

        if let Some(name) = self.show {
            return Some(Commands::Bundle(bundle::Args {
                command: bundle::Commands::Show { name },
            }));
        }

        if let Some(name) = self.edit {
            return Some(Commands::Bundle(bundle::Args {
                command: bundle::Commands::Edit { name },
            }));
        }

        if let Some(name) = self.delete {
            return Some(Commands::Bundle(bundle::Args {
                command: bundle::Commands::Delete { name },
            }));
        }

        if let Some(name) = self.search {
            return Some(Commands::Bundle(bundle::Args {
                command: bundle::Commands::Search { name },
            }));
        }

        if let Some(name) = self.add {
            return Some(Commands::Bundle(bundle::Args {
                command: bundle::Commands::Add { name },
            }));
        }

        if let Some(name) = self.generate {
            return Some(Commands::Password(password::Args {
                command: password::Commands::Add { name },
            }));
        }

        self.command
    }
}

#[tokio::main]
pub async fn main() -> BazaR<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    cleanup_tmp_folder().map_err(|e| {
        exn::Exn::new(baza_core::error::Error::Message(format!(
            "Failed to cleanup temporary folder: {}",
            e
        )))
    })?;

    let fmt = fmt::layer()
        .with_target(false)
        .without_time()
        .compact()
        .json()
        .with_filter(filter);
    tracing_subscriber::registry().with(fmt).init();

    let args = Cli::parse();

    if let Some(str) = args.stdin.as_ref() {
        container::from_stdin(str.clone()).map_err(|e| {
            exn::Exn::new(baza_core::error::Error::Message(format!(
                "Failed to create bundle from STDIN: {}",
                e
            )))
        })?;
        return Ok(());
    };

    match args.get_command() {
        Some(cmd) => {
            if let Err(err) = cmd.run() {
                tracing::error!(?err);
                std::process::exit(1);
            }
        }
        None => {
            Cli::command().print_long_help().ok();
        }
    }

    Ok(())
}
