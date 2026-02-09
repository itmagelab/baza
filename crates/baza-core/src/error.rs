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

    #[error(transparent)]
    WalkDir(#[from] walkdir::Error),

    #[error(transparent)]
    Regex(#[from] regex_lite::Error),

    #[error(transparent)]
    AesGcm(#[from] aes_gcm::Error),

    #[error(transparent)]
    Arboard(#[from] arboard::Error),

    #[error(transparent)]
    StripPrefix(#[from] std::path::StripPrefixError),

    #[error(transparent)]
    Env(#[from] std::env::VarError),

    #[error(transparent)]
    RedbDatabase(#[from] redb::DatabaseError),

    #[error(transparent)]
    RedbTransaction(#[from] redb::TransactionError),

    #[error(transparent)]
    RedbTable(#[from] redb::TableError),

    #[error(transparent)]
    RedbStorage(#[from] redb::StorageError),

    #[error(transparent)]
    RedbCommit(#[from] redb::CommitError),

    #[error(transparent)]
    Persist(#[from] tempfile::PersistError),

    #[error("{0}")]
    Any(String),
}
