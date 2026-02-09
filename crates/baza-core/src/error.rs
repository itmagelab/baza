use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Message(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Toml(#[from] toml::ser::Error),

    #[error(transparent)]
    TomlDe(#[from] toml::de::Error),

    #[cfg(not(target_arch = "wasm32"))]
    #[error(transparent)]
    WalkDir(#[from] walkdir::Error),

    #[error(transparent)]
    Regex(#[from] regex_lite::Error),

    #[error(transparent)]
    AesGcm(#[from] aes_gcm::Error),

    #[cfg(not(target_arch = "wasm32"))]
    #[error(transparent)]
    Arboard(#[from] arboard::Error),

    #[error(transparent)]
    StripPrefix(#[from] std::path::StripPrefixError),

    #[cfg(not(target_arch = "wasm32"))]
    #[error(transparent)]
    Env(#[from] std::env::VarError),

    #[cfg(not(target_arch = "wasm32"))]
    #[error(transparent)]
    RedbDatabase(#[from] redb::DatabaseError),

    #[cfg(not(target_arch = "wasm32"))]
    #[error(transparent)]
    RedbTransaction(#[from] redb::TransactionError),

    #[cfg(not(target_arch = "wasm32"))]
    #[error(transparent)]
    RedbTable(#[from] redb::TableError),

    #[cfg(not(target_arch = "wasm32"))]
    #[error(transparent)]
    RedbStorage(#[from] redb::StorageError),

    #[cfg(not(target_arch = "wasm32"))]
    #[error(transparent)]
    RedbCommit(#[from] redb::CommitError),

    #[cfg(not(target_arch = "wasm32"))]
    #[error(transparent)]
    Persist(#[from] tempfile::PersistError),

    #[error("{0}")]
    Any(String),
}
