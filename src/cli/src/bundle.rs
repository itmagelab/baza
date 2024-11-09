use core::container;

use clap::{command, Args as ClapArgs, Subcommand};

#[derive(Debug, ClapArgs)]
pub(crate) struct Args {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    Create { name: String },
}

pub(crate) fn handle(args: Args) {
    match args.command {
        Commands::Create { name } => {
            container::create(name);
        }
    }
}
