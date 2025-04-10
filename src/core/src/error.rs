use std::path::StripPrefixError;

use tempfile::PersistError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("No name found for save")]
    NoName,    
    #[error("Failed to open editor.")]
    OpenEditor,    
    #[error("Path already exists.")]
    PathExists,    
    #[error("Size must be greater than zero.")]
    ZeroSize,    
    #[error("At least one of these must be specified: letters, numbers, symbols.")]
    MustSpecifyAtLeastOne,    
    #[error("Too few arguments provided.")]
    TooFewArguments,    
    #[error("Encryption failed.")]
    EncryptionError(aes_gcm::Error),    
    #[error("At least one box is required (e.g., 'work::login').")]
    AtLeastOneBoxRequired,    
    #[error("Bundle '{0}' does not exist.")]
    BundleNotExist(String),
    #[error("Box '{box}' has no bundles.")]
    BundlesIsEmpty { r#box: String },    
    #[error("No key found. Use 'baza unlock' or 'baza init'.")]
    KeyNotFound,    
    #[error("Failed to clean up temporary folder: {0}. Ensure it is writable.")]
    CleanupTmpFolder(std::io::Error),
    #[error("Decryption failed: {0}.")]
    Decryption(aes_gcm::Error),    
    #[error("Encryption error: {0}")]
    Encryption(aes_gcm::Error),    
    #[error("No pointer found in the data.")]
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
