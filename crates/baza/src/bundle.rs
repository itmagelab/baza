use baza_core::{container, BazaR};

use clap::{Args as ClapArgs, Subcommand};

#[derive(Debug, ClapArgs)]
pub(crate) struct Args {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    /// Create bundle of passwords
    Add { name: String },
    /// Generate a new bundle
    Generate { name: String },
    /// Edit exists bundle of passwords
    Edit { name: String },
    /// Deleting a bundle
    Delete { name: String },
    /// Search bundle by name
    Search { name: String },
    /// Copy all bundle to clipboard
    Copy { name: String },
    /// Show content of bundle
    Show { name: String },
}

pub(crate) fn handle(args: Args) -> BazaR<()> {
    match args.command {
        Commands::Add { name } => {
            container::add(name, None)?;
        }
        Commands::Generate { name } => {
            container::generate(name)?;
        }
        Commands::Delete { name } => {
            container::delete(name)?;
        }
        Commands::Edit { name } => {
            container::update(name)?;
        }
        Commands::Show { name } => {
            container::read(name)?;
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
