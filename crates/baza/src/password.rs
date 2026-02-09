use argh::FromArgs;
use baza_core::{container, generate, BazaR};
use std::io::{self, Write};

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "password")]
/// Generating a password
pub(crate) struct Args {
    #[argh(subcommand)]
    pub(crate) command: SubCommands,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand)]
pub(crate) enum SubCommands {
    Generate(GenerateArgs),
    Add(AddArgs),
}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "generate")]
/// Generating a password
pub(crate) struct GenerateArgs {
    /// length of the password
    #[argh(option, default = "24")]
    pub(crate) length: usize,

    /// exclude letters
    #[argh(switch)]
    pub(crate) no_letters: bool,

    /// exclude symbols
    #[argh(switch)]
    pub(crate) no_symbols: bool,

    /// exclude numbers
    #[argh(switch)]
    pub(crate) no_numbers: bool,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "add")]
/// Add a new password bundle
pub(crate) struct AddArgs {
    #[argh(positional)]
    pub(crate) name: String,
}

pub(crate) fn handle(args: Args) -> BazaR<()> {
    match args.command {
        SubCommands::Add(args) => {
            container::generate(args.name)?;
            Ok(())
        }
        SubCommands::Generate(args) => {
            let mut stdout = io::stdout();
            if args.no_letters && args.no_symbols && args.no_numbers {
                exn::bail!(baza_core::error::Error::Message(
                    "at least one character type must be enabled".into()
                ));
            };
            writeln!(
                stdout,
                "{}",
                generate(
                    args.length,
                    args.no_letters,
                    args.no_symbols,
                    args.no_numbers
                )?
            )
            .map_err(|e| exn::Exn::new(baza_core::error::Error::Io(e)))?;
            Ok(())
        }
    }
}
