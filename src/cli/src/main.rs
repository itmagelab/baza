use clap::{Parser, Subcommand};

mod bundle;
mod password;

#[derive(Debug, Subcommand)]
pub(crate) enum Commands {
    Bundle(bundle::Args),
    Password(password::Args),
}

#[derive(Parser, Debug)]
#[command(name = "baza")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
pub async fn main() {
    let args = Cli::parse();
    let result = match args.command {
        Commands::Password(s) => password::handle(s),
        Commands::Bundle(s) => {
            bundle::handle(s);
            Ok(())
        }
    };
    match result {
        Ok(_) => (),
        Err(err) => {
            eprintln!("{}", err);
            std::process::exit(1);
        }
    };
}
