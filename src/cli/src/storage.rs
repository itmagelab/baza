use core::error::Error;

use clap::{command, Args as ClapArgs, Subcommand};

#[derive(Debug, ClapArgs)]
pub(crate) struct Args {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    Init,
}

pub(crate) fn handle(args: Args) -> Result<(), Error> {
    match args.command {
        Commands::Init => Ok(()),
    }
}
