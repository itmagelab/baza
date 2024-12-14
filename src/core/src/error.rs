use tempfile::PersistError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("This is a common error")]
    CommonBazaError,
    #[error("Errror persisting file: {0}")]
    TempBazaError(PersistError),
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
}
