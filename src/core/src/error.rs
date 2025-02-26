use std::path::StripPrefixError;

use tempfile::PersistError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
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
    #[error("arboard error")]
    ArboardError(arboard::Error),
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
}
