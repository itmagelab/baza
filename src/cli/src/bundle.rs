use baza_core::{container, BazaR};

use clap::{command, Args as ClapArgs, Subcommand};

#[derive(Debug, ClapArgs)]
pub(crate) struct Args {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    Create { name: String },
    Delete { name: String },
    Edit { name: String },
    Search { name: String },
    Copy { name: String },
}

pub(crate) fn handle(args: Args) -> BazaR<()> {
    match args.command {
        Commands::Create { name } => {
            container::create(name)?;
        }
        Commands::Delete { name } => {
            container::delete(name)?;
        }
        Commands::Edit { name } => {
            container::edit(name)?;
        }
        Commands::Search { name } => {
            container::search(name)?;
        }
        Commands::Copy { name } => {
            container::copy_to_clipboard(name)?;
        }
    };
    Ok(())
}
