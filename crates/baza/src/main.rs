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

    /// passphrase for the database
    #[argh(option)]
    passphrase: Option<String>,

    /// TOTP code for database unlock
    #[argh(option, short = 't')]
    totp: Option<String>,

    /// list all containers
    #[argh(switch, short = 'l')]
    list: bool,

    #[argh(subcommand)]
    command: Option<Commands>,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand)]
enum Commands {
    Init(InitArgs),
    Bundle(bundle::Args),
    Password(password::Args),
    List(ListArgs),
    Version(VersionArgs),
    Dump(DumpArgs),
    Restore(RestoreArgs),
    Totp(TotpArgs),
    Unlock(UnlockArgs),
    Lock(LockArgs),
    #[cfg(feature = "s3")]
    Push(PushArgs),
    #[cfg(feature = "s3")]
    Pull(PullArgs),
}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "init")]
/// Initializing the database
struct InitArgs {
    /// passphrase for the database
    #[argh(option, short = 'p')]
    passphrase: Option<String>,

    /// force overwrite of existing database without confirmation prompt
    #[argh(switch, short = 'f')]
    force: bool,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "list")]
/// List all containers
struct ListArgs {}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "version")]
/// Show Version
struct VersionArgs {}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "dump")]
/// Dump database to file
struct DumpArgs {}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "restore")]
/// Restore database from file
struct RestoreArgs {
    /// path to dump file
    #[argh(positional)]
    path: String,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "unlock")]
/// Verify credentials and print export command for BAZA_PASSPHRASE
struct UnlockArgs {
    /// passphrase for the database
    #[argh(option, short = 'p')]
    passphrase: Option<String>,

    /// TOTP code for database unlock
    #[argh(option, short = 't')]
    totp: Option<String>,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "lock")]
/// Print unset command for BAZA_PASSPHRASE
struct LockArgs {}

#[cfg(feature = "s3")]
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "push")]
/// Push database to S3
struct PushArgs {}

#[cfg(feature = "s3")]
#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "pull")]
/// Pull database from S3
struct PullArgs {}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "totp")]
/// Manage TOTP authentication
struct TotpArgs {
    #[argh(subcommand)]
    command: TotpSubCommands,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand)]
enum TotpSubCommands {
    Enable(TotpEnableArgs),
    Disable(TotpDisableArgs),
    Status(TotpStatusArgs),
}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "enable")]
/// Enable TOTP authentication
struct TotpEnableArgs {
    /// do not print QR code to terminal
    #[argh(switch)]
    no_qr: bool,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "disable")]
/// Disable TOTP authentication
struct TotpDisableArgs {}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "status")]
/// Show TOTP status
struct TotpStatusArgs {}
fn run_command(cmd: Commands) -> BazaR<()> {
    match cmd {
        Commands::Password(s) => password::handle(s)?,
        Commands::Bundle(s) => bundle::handle(s)?,
        Commands::Init(args) => {
            use colored::Colorize;
            if pollster::block_on(baza_core::storage::is_initialized())? {
                if !args.force {
                    let datadir = &baza_core::Config::get().main.datadir;
                    eprint!(
                        "Warning: A Baza vault already exists at: {}\nDo you really want to overwrite it and delete all existing data? [y/N]: ",
                        datadir
                    );
                    std::io::Write::flush(&mut std::io::stderr()).ok();
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).or_raise(|| {
                        baza_core::error::Error::Message("Failed to read input".into())
                    })?;
                    let input = input.trim().to_lowercase();
                    if input != "y" && input != "yes" {
                        println!("Initialization aborted.");
                        return Ok(());
                    }
                }
            }
            let p = pollster::block_on(baza_core::init(args.passphrase))?;

            println!("\n{}", "=============================================================".bright_yellow());
            println!("{}", "             CRITICAL SECURITY WARNING".bright_red().bold());
            println!("{}", "=============================================================".bright_yellow());
            println!(" Please save the following master passphrase.");
            println!(" You will need it to unlock your vault in the future.");
            println!(" Baza does not store this key, so it CANNOT be recovered!");
            println!("");
            println!(" Master Passphrase:");
            println!(" *  {}", p.bright_green().bold());
            println!("{}", "=============================================================".bright_yellow());
            println!(" {}\n", "Vault initialized successfully!".bright_green());
        }
        Commands::List(_) => {
            pollster::block_on(container::search(String::from(".*")))?;
        }
        Commands::Version(_) => {
            println!("{}", env!("CARGO_PKG_VERSION"));
        }
        Commands::Dump(_) => {
            handle_dump()?;
        }
        Commands::Restore(args) => {
            handle_restore(args.path)?;
        }
        #[cfg(feature = "s3")]
        Commands::Push(_) => {
            baza_core::s3::push()?;
        }
        #[cfg(feature = "s3")]
        Commands::Pull(_) => {
            baza_core::s3::pull()?;
        }
        Commands::Totp(args) => match args.command {
            TotpSubCommands::Enable(enable_args) => {
                let (secret, url, _) = pollster::block_on(baza_core::totp::enable())?;
                println!("TOTP enabled successfully!");
                println!("Secret key (Base32): {}", secret);
                println!("OTPAuth URL: {}", url);
                if !enable_args.no_qr {
                    println!("\nScan this QR code with your authenticator app:\n");
                    let code = qrcode::QrCode::new(&url).map_err(|e| {
                        baza_core::error::Error::Message(format!(
                            "Failed to generate QR code: {}",
                            e
                        ))
                    })?;
                    let image = code
                        .render::<qrcode::render::unicode::Dense1x2>()
                        .dark_color(qrcode::render::unicode::Dense1x2::Light)
                        .light_color(qrcode::render::unicode::Dense1x2::Dark)
                        .build();
                    println!("{}", image);
                }
            }
            TotpSubCommands::Disable(_) => {
                pollster::block_on(baza_core::totp::disable())?;
                println!("TOTP authentication disabled.");
            }
            TotpSubCommands::Status(_) => {
                let enabled = pollster::block_on(baza_core::totp::is_enabled())?;
                if enabled {
                    println!("TOTP authentication is enabled.");
                } else {
                    println!("TOTP authentication is disabled.");
                }
            }
        },
        Commands::Unlock(args) => {
            let passphrase_opt = args
                .passphrase
                .or_else(|| std::env::var("BAZA_PASSPHRASE").ok());
            let totp_opt = args.totp.or_else(|| std::env::var("BAZA_TOTP").ok());

            let (passphrase, totp_code) = acquire_credentials(passphrase_opt, totp_opt)?;
            pollster::block_on(baza_core::unlock(passphrase.clone(), totp_code))?;
            println!("export BAZA_PASSPHRASE=\"{}\"", passphrase);
        }
        Commands::Lock(_) => {
            println!("unset BAZA_PASSPHRASE");
        }
    };
    Ok(())
}

fn main() -> BazaR<()> {
    simple_logger::init_with_level(log::Level::Info).ok();

    if let Err(err) = run_main() {
        eprintln!("Error: {}", err);
        std::process::exit(1);
    }
    Ok(())
}

fn run_main() -> BazaR<()> {
    let config_path = if let Ok(path) = std::env::var("BAZA_CONFIG") {
        std::path::PathBuf::from(path)
    } else {
        let default_path = baza_core::Config::default_config_path()?;
        let mut chosen_path = default_path.clone();

        if !cfg!(debug_assertions) && !default_path.exists() {
            // In production, try to find legacy configuration if standard one is missing
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            let legacy_paths = [
                format!("{}/.baza/config.toml", home),
                format!("{}/.Baza.toml", home),
            ];

            for path in legacy_paths.iter() {
                let p = std::path::PathBuf::from(path);
                if p.exists() {
                    chosen_path = p;
                    break;
                }
            }
        }
        chosen_path
    };

    // Default or found modern config path
    baza_core::Config::build(&config_path)?;

    handle_args()
}

fn acquire_credentials(
    passphrase_opt: Option<String>,
    totp_opt: Option<String>,
) -> BazaR<(String, Option<String>)> {
    let passphrase = match passphrase_opt {
        Some(p) => p,
        None => {
            eprint!("Enter passphrase: ");
            std::io::Write::flush(&mut std::io::stderr()).ok();
            let mut input = String::new();
            std::io::stdin().read_line(&mut input).or_raise(|| {
                baza_core::error::Error::Message("Failed to read passphrase".into())
            })?;
            input.trim().to_string()
        }
    };

    let totp_enabled = pollster::block_on(baza_core::totp::is_enabled()).unwrap_or(false);
    let totp_code = if totp_enabled {
        match totp_opt {
            Some(c) => Some(c),
            None => {
                let uuid = pollster::block_on(baza_core::totp::get_uuid())
                    .unwrap_or_else(|_| "default".to_string());
                eprintln!("TOTP code required (ID: {})", uuid);
                eprint!("Enter TOTP code: ");
                std::io::Write::flush(&mut std::io::stderr()).ok();
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).or_raise(|| {
                    baza_core::error::Error::Message("Failed to read TOTP code".into())
                })?;
                Some(input.trim().to_string())
            }
        }
    } else {
        totp_opt
    };

    Ok((passphrase, totp_code))
}

fn handle_args() -> BazaR<()> {
    cleanup_tmp_folder().or_raise(|| {
        baza_core::error::Error::Message("Failed to cleanup temporary folder".into())
    })?;

    let args: Cli = argh::from_env();

    // Handle passphrase acquisition
    let passphrase = args
        .passphrase
        .or_else(|| std::env::var("BAZA_PASSPHRASE").ok());

    let totp_code = args.totp.or_else(|| std::env::var("BAZA_TOTP").ok());

    let should_unlock = if let Some(cmd) = &args.command {
        #[cfg(feature = "s3")]
        let is_s3 = matches!(cmd, Commands::Push(_) | Commands::Pull(_));
        #[cfg(not(feature = "s3"))]
        let is_s3 = false;

        let is_password_generate = match cmd {
            Commands::Password(p_args) => {
                matches!(p_args.command, password::SubCommands::Generate(_))
            }
            _ => false,
        };

        !matches!(
            cmd,
            Commands::Init(_) | Commands::Version(_) | Commands::Unlock(_) | Commands::Lock(_)
        ) && !is_s3
            && !is_password_generate
    } else {
        true
    };

    if should_unlock {
        let (passphrase, totp_code) = acquire_credentials(passphrase, totp_code)?;
        pollster::block_on(baza_core::unlock(passphrase, totp_code))?;
    }

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
            run_command(cmd)?;
        }
        None => {
            println!("Baza: The base password manager");
            println!("Use --help for usage information");
        }
    }

    Ok(())
}

fn handle_dump() -> BazaR<()> {
    use exn::ResultExt;
    use std::fs::File;
    use std::io::Write;

    let data = pollster::block_on(baza_core::storage::dump())?;
    let dumped = baza_core::dump::dump(&data, baza_core::dump::Algorithm::Lz4)
        .or_raise(|| baza_core::error::Error::Message("Failed to dump database".into()))?;

    let mut file = File::create("dump.baza")
        .or_raise(|| baza_core::error::Error::Message("Failed to create dump file".into()))?;
    file.write_all(&dumped)
        .or_raise(|| baza_core::error::Error::Message("Failed to write dump file".into()))?;

    println!("Database dumped to dump.baza");
    Ok(())
}

fn handle_restore(path: String) -> BazaR<()> {
    use exn::ResultExt;
    use std::fs;

    let data = fs::read(path)
        .or_raise(|| baza_core::error::Error::Message("Failed to read dump file".into()))?;
    let restored = baza_core::dump::restore::<Vec<(String, Vec<u8>)>>(&data)
        .or_raise(|| baza_core::error::Error::Message("Failed to restore database".into()))?;

    pollster::block_on(baza_core::storage::restore(restored))?;

    println!("Database restored from dump");
    Ok(())
}
