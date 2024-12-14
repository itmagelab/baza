use core::{container, BazaR};

use clap::{command, Args as ClapArgs, Subcommand};

#[derive(Debug, ClapArgs)]
pub(crate) struct Args {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    Create { name: String },
    Edit { name: String },
}

pub(crate) fn handle(args: Args) -> BazaR<()> {
    match args.command {
        Commands::Create { name } => {
            container::create(name)?;
        }
        Commands::Edit { name } => {
            container::edit(name)?;
        }
    };
    Ok(())
}
