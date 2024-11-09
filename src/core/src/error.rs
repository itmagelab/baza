use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
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
}
