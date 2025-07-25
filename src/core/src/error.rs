use std::path::StripPrefixError;

use tempfile::PersistError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("No name found for save")]
    NoName,
    #[error("This is a common error")]
    CommonBazaError,
    #[error("Error opening editor")]
    OpenEditor,
    #[error("Path already exists")]
    PathExists,
    #[error("Size must be greater than zero")]
    ZeroSize,
    #[error("You must specify at least one of the following: latters, numbers, symbols")]
    MustSpecifyAtLeastOne,
    #[error("Too few arguments")]
    TooFewArguments,
    #[error("Encription error")]
    EncriptionError(aes_gcm::Error),
    #[error("At least one box is required, but none were found. Like: `work::login`")]
    AtLeastOneBoxRequired,
    #[error("Bundle {0} does not exist")]
    BundleNotExist(String),
    #[error("The box {box} have not bundles")]
    BundlesIsEmpty { r#box: String },
    #[error("No key found. Try using the command `baza unlock` or `baza init`")]
    KeyNotFound,
    #[error("Can't cleanup tmp folder, must be a writable: {0}")]
    CleanupTmpFolder(std::io::Error),
    #[error("Decription error: {0}")]
    Decription(aes_gcm::Error),
    #[error("Encription error: {0}")]
    Encription(aes_gcm::Error),
    #[error("No pointer found.")]
    NoPointerFound,
    // From traits
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    Walkdir(#[from] walkdir::Error),
    #[error(transparent)]
    StripPrefix(#[from] StripPrefixError),
    #[error(transparent)]
    Persist(#[from] PersistError),
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
    #[error(transparent)]
    Git2(#[from] git2::Error),
    #[error(transparent)]
    EnvVar(#[from] std::env::VarError),
    #[error(transparent)]
    Regex(#[from] regex::Error),
    #[error(transparent)]
    Arboard(#[from] arboard::Error),
    #[error(transparent)]
    GixCommit(#[from] gix::commit::Error),
    #[error(transparent)]
    GixObject(#[from] gix::object::write::Error),
    #[error(transparent)]
    GixConfig(#[from] Box<gix::config::Error>),
    #[error(transparent)]
    GixConfigFile(#[from] gix::config::file::set_raw_value::Error),
    #[error(transparent)]
    GixInit(#[from] Box<gix::init::Error>),
    #[error(transparent)]
    GixDiscover(Box<gix::discover::Error>),
    #[error(transparent)]
    GixRefFind(#[from] gix::reference::find::existing::Error),
    #[error(transparent)]
    GixRefHeadTree(#[from] gix::reference::head_tree::Error),
    #[error(transparent)]
    GixOpen(#[from] Box<gix::open::Error>),
    #[error(transparent)]
    GixWorktreeOpenIndex(#[from] Box<gix::worktree::open_index::Error>),
    #[error(transparent)]
    GixOdbStoreLoadIndex(#[from] gix::odb::store::load_index::Error),
    #[error(transparent)]
    GixRefHeadId(#[from] gix::reference::head_id::Error),
    #[error(transparent)]
    GixObjTryInto(#[from] gix::object::try_into::Error),
    #[error(transparent)]
    GixObjFindExisting(#[from] gix::object::find::existing::Error),
    #[error(transparent)]
    GixObjCommit(#[from] gix::object::commit::Error),
}

impl From<gix::config::Error> for Error {
    fn from(err: gix::config::Error) -> Self {
        Error::GixConfig(Box::new(err))
    }
}

impl From<gix::init::Error> for Error {
    fn from(err: gix::init::Error) -> Self {
        Error::GixInit(Box::new(err))
    }
}

impl From<gix::open::Error> for Error {
    fn from(err: gix::open::Error) -> Self {
        Error::GixOpen(Box::new(err))
    }
}

impl From<gix::worktree::open_index::Error> for Error {
    fn from(err: gix::worktree::open_index::Error) -> Self {
        Error::GixWorktreeOpenIndex(Box::new(err))
    }
}
