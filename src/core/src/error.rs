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
    #[error("IO error: {0}")]
    IO(#[from] std::io::Error),
    #[error("Size must be greater than zero")]
    ZeroSize,
    #[error("You must specify at least one of the following: latters, numbers, symbols")]
    MustSpecifyAtLeastOne,
    #[error("Too few arguments")]
    TooFewArguments,

    // From traits
    #[error("walkdir::Error: {0}")]
    WalkdirError(#[from] walkdir::Error),
    #[error("StripPrefixError: {0}")]
    StripPrefixError(#[from] StripPrefixError),
    #[error("PersistError: {0}")]
    PersistErrorError(#[from] PersistError),
    #[error("anyhow::Error: {0}")]
    AnyhowError(#[from] anyhow::Error),
}
