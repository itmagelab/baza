use baza_core::{container, BazaR};

use clap::{command, Args as ClapArgs, Subcommand};

#[derive(Debug, ClapArgs)]
pub(crate) struct Args {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    /// Create bundle of passwords
    Create { name: String },
    /// Edit exists bundle of passwords
    Edit { name: String },
    /// Deleting a bundle
    Delete { name: String },
    /// Search bundle by name
    Search { name: String },
    /// Copy all bundle to clipboard
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
