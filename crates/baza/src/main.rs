use argh::FromArgs;
use baza_core::{cleanup_tmp_folder, container, BazaR};
use exn::ResultExt;

mod bundle;
mod password;

#[derive(FromArgs, Debug)]
/// Baza: The base password manager
struct Cli {
    /// adding bundle of passwords
    #[argh(option, short = 'a')]
    add: Option<String>,

    /// create bundle from STDIN
    #[argh(option)]
    stdin: Option<String>,

    /// generate default bundle
    #[argh(option, short = 'g')]
    generate: Option<String>,

    /// edit exists bundle of passwords
    #[argh(option, short = 'e')]
    edit: Option<String>,

    /// deleting a bundle
    #[argh(option, short = 'd')]
    delete: Option<String>,

    /// search bundle by name
    #[argh(option, short = 's')]
    search: Option<String>,

    /// copy all bundle to clipboard
    #[argh(option, short = 'c')]
    copy: Option<String>,

    /// show content of bundle
    #[argh(option, short = 'p')]
    show: Option<String>,

    /// list all containers
    #[argh(switch, short = 'l')]
    list: bool,

    #[argh(subcommand)]
    command: Option<Commands>,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand)]
enum Commands {
    Unlock(UnlockArgs),
    Lock(LockArgs),
    Init(InitArgs),
    Bundle(bundle::Args),
    Password(password::Args),
    List(ListArgs),
    Version(VersionArgs),
}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "unlock")]
/// Unlock database
struct UnlockArgs {}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "lock")]
/// Lock database
struct LockArgs {}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "init")]
/// Initializing the database
struct InitArgs {
    /// passphrase for the database
    #[argh(option, short = 'p')]
    passphrase: Option<String>,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "list")]
/// List all containers
struct ListArgs {}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "version")]
/// Show Version
struct VersionArgs {}

fn run_command(cmd: Commands) -> BazaR<()> {
    match cmd {
        Commands::Password(s) => password::handle(s)?,
        Commands::Bundle(s) => bundle::handle(s)?,
        Commands::Init(args) => {
            if pollster::block_on(baza_core::storage::is_initialized())? {
                println!("Warning: Vault already exists.");
            }
            let p = baza_core::init(args.passphrase)?;
            println!("Vault initialized with passphrase: {}", p);
        }
        Commands::Unlock(_) => baza_core::unlock(None)?,
        Commands::Lock(_) => baza_core::lock()?,
        Commands::List(_) => {
            pollster::block_on(container::search(String::from(".*")))?;
        }
        Commands::Version(_) => {
            println!("{}", env!("CARGO_PKG_VERSION"));
        }
    };
    Ok(())
}

fn main() -> BazaR<()> {
    simple_logger::init_with_level(log::Level::Info).ok();

    let home = std::env::var("HOME").or_raise(|| {
        baza_core::error::Error::Message("Failed to get HOME environment variable".into())
    })?;

    let config_paths = [
        format!("{}/.config/baza/baza.toml", home),
        format!("{}/.baza/config.toml", home),
        format!("{}/.Baza.toml", home),
    ];

    let mut found_config = false;
    for path in config_paths.iter() {
        let p = std::path::PathBuf::from(path);
        if p.exists() {
            baza_core::Config::build(&p)?;
            found_config = true;
            break;
        }
    }

    if !found_config {
        let default_path = std::path::PathBuf::from(&config_paths[0]);
        baza_core::Config::build(&default_path)?;
    }

    cleanup_tmp_folder().or_raise(|| {
        baza_core::error::Error::Message("Failed to cleanup temporary folder".into())
    })?;

    let args: Cli = argh::from_env();

    if let Some(str) = args.stdin {
        pollster::block_on(container::from_stdin(str)).or_raise(|| {
            baza_core::error::Error::Message("Failed to create bundle from STDIN".into())
        })?;
        return Ok(());
    };

    if args.list {
        return run_command(Commands::List(ListArgs {}));
    }

    if let Some(name) = args.copy {
        return bundle::handle(bundle::Args {
            command: bundle::SubCommands::Copy(bundle::CopyArgs { name }),
        });
    }

    if let Some(name) = args.show {
        return bundle::handle(bundle::Args {
            command: bundle::SubCommands::Show(bundle::ShowArgs { name }),
        });
    }

    if let Some(name) = args.edit {
        return bundle::handle(bundle::Args {
            command: bundle::SubCommands::Edit(bundle::EditArgs { name }),
        });
    }

    if let Some(name) = args.delete {
        return bundle::handle(bundle::Args {
            command: bundle::SubCommands::Delete(bundle::DeleteArgs { name }),
        });
    }

    if let Some(name) = args.search {
        return bundle::handle(bundle::Args {
            command: bundle::SubCommands::Search(bundle::SearchArgs { name }),
        });
    }

    if let Some(name) = args.add {
        return bundle::handle(bundle::Args {
            command: bundle::SubCommands::Add(bundle::AddArgs { name }),
        });
    }

    if let Some(name) = args.generate {
        return password::handle(password::Args {
            command: password::SubCommands::Add(password::AddArgs { name }),
        });
    }

    match args.command {
        Some(cmd) => {
            if let Err(err) = run_command(cmd) {
                tracing::error!(?err);
                std::process::exit(1);
            }
        }
        None => {
            println!("Baza: The base password manager");
            println!("Use --help for usage information");
        }
    }

    Ok(())
}
