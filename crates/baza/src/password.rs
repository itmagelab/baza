use baza_core::{container, generate, BazaR};
use std::io::{self, Write};

use clap::{command, value_parser, Args as ClapArgs, Subcommand};

#[derive(Debug, ClapArgs)]
pub(crate) struct Args {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    Generate {
        #[arg(value_parser = value_parser!(u8).range(1..=255))]
        length: u8,
        #[arg(long, default_value_t = false)]
        no_latters: bool,
        #[arg(long, default_value_t = false)]
        no_symbols: bool,
        #[arg(long, default_value_t = false)]
        no_numbers: bool,
    },
    Add {
        name: String,
    },
}

pub(crate) fn handle(args: Args) -> BazaR<()> {
    match args.command {
        Commands::Add { name } => {
            container::generate(name)?;
            Ok(())
        }
        Commands::Generate {
            length,
            no_latters,
            no_symbols,
            no_numbers,
        } => {
            let mut stdout = io::stdout();
            if no_latters && no_symbols && no_numbers {
                anyhow::bail!(
                    "at least one of --no-latters, --no-symbols or --no-numbers must be specified"
                );
            };
            writeln!(
                stdout,
                "{}",
                generate(length, no_latters, no_symbols, no_numbers)?
            )?;
            Ok(())
        }
    }
}
