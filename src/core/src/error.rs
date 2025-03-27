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
    #[error("Must be more what one box; like: `work::login`")]
    BoxMoreOne,
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
    GixConfig(Box<gix::config::Error>),
    #[error(transparent)]
    GixConfigFile(#[from] gix::config::file::set_raw_value::Error),
    #[error(transparent)]
    GixInit(Box<gix::init::Error>),
    #[error(transparent)]
    GixDiscover(Box<gix::discover::Error>),
    #[error(transparent)]
    GixRefFind(#[from] gix::reference::find::existing::Error),
    #[error(transparent)]
    GixRefHeadTree(#[from] gix::reference::head_tree::Error),
}
