use argh::FromArgs;
use baza_core::{container, BazaR};

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "bundle")]
/// Work with passwords (bundles)
pub(crate) struct Args {
    #[argh(subcommand)]
    pub(crate) command: SubCommands,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand)]
pub(crate) enum SubCommands {
    Add(AddArgs),
    Generate(GenerateArgs),
    Edit(EditArgs),
    Delete(DeleteArgs),
    Search(SearchArgs),
    Copy(CopyArgs),
    Show(ShowArgs),
}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "add")]
/// Create bundle of passwords
pub(crate) struct AddArgs {
    #[argh(positional)]
    pub(crate) name: String,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "generate")]
/// Generate a new bundle
pub(crate) struct GenerateArgs {
    #[argh(positional)]
    pub(crate) name: String,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "edit")]
/// Edit exists bundle of passwords
pub(crate) struct EditArgs {
    #[argh(positional)]
    pub(crate) name: String,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "delete")]
/// Deleting a bundle
pub(crate) struct DeleteArgs {
    #[argh(positional)]
    pub(crate) name: String,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "search")]
/// Search bundle by name
pub(crate) struct SearchArgs {
    #[argh(positional)]
    pub(crate) name: String,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "copy")]
/// Copy all bundle to clipboard
pub(crate) struct CopyArgs {
    #[argh(positional)]
    pub(crate) name: String,
}

#[derive(FromArgs, Debug)]
#[argh(subcommand, name = "show")]
/// Show content of bundle
pub(crate) struct ShowArgs {
    #[argh(positional)]
    pub(crate) name: String,
}

pub(crate) fn handle(args: Args) -> BazaR<()> {
    match args.command {
        SubCommands::Add(args) => {
            pollster::block_on(container::add(args.name, None))?;
        }
        SubCommands::Generate(args) => {
            pollster::block_on(container::generate(args.name))?;
        }
        SubCommands::Delete(args) => {
            pollster::block_on(container::delete(args.name))?;
        }
        SubCommands::Edit(args) => {
            pollster::block_on(container::update(args.name))?;
        }
        SubCommands::Show(args) => {
            pollster::block_on(container::read(args.name))?;
        }
        SubCommands::Search(args) => {
            pollster::block_on(container::search(args.name))?;
        }
        SubCommands::Copy(args) => {
            pollster::block_on(container::copy_to_clipboard(args.name))?;
        }
    };
    Ok(())
}
