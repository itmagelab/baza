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
    #[error("Help error: {0}")]
    HelpError(std::io::Error),
    #[error("The box {box} have not bundles")]
    BundlesIsEmpty { r#box: String },
    #[error("No key found. Try using the command `baza unlock` or `baza init`")]
    KeyNotFound,
    #[error("Can't cleanup tmp folder, must be a writable: {0}")]
    CleanupTmpFolder(std::io::Error),
    // From traits
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("walkdir::Error: {0}")]
    WalkdirError(#[from] walkdir::Error),
    #[error("StripPrefixError: {0}")]
    StripPrefixError(#[from] StripPrefixError),
    #[error("PersistError: {0}")]
    PersistErrorError(#[from] PersistError),
    #[error("anyhow::Error: {0}")]
    AnyhowError(#[from] anyhow::Error),
    #[error("git2::Error: {0}")]
    GitError(#[from] git2::Error),
    #[error("env::Error: {0}")]
    EnvError(#[from] std::env::VarError),
    #[error("Decription error: {0}")]
    Decription(aes_gcm::Error),
    #[error("Encription error: {0}")]
    Encription(aes_gcm::Error),
}
