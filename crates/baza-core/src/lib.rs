//! # Baza
//!
//! The core library for crate Baza crate
//!

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use colored::Colorize;
use core::str;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tracing::instrument;
use uuid::Uuid;

use rand::Rng;

pub mod r#box;
pub mod bundle;
pub mod container;
pub mod error;
pub mod storage;

pub static CONFIG: OnceLock<Config> = OnceLock::new();
pub const TTL_SECONDS: u64 = 15;
pub const DEFAULT_AUTHOR: &str = "Baza";
pub const DEFAULT_EMAIL: &str = "baza@itmagelab.com";

pub type BazaR<T> = Result<T, exn::Exn<error::Error>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub main: MainConfig,
    pub gitfs: GitFsConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MainConfig {
    pub datadir: String,
    pub box_delimiter: String,
    pub bundle_delimiter: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GitFsConfig {
    pub url: Option<String>,
    pub privatekey: Option<String>,
    pub passphrase: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageConfig {
    pub r#type: Type,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Type {
    #[serde(rename = "gitfs")]
    Gitfs,
    #[serde(rename = "redb")]
    Redb,
}

impl Default for Config {
    fn default() -> Self {
        let home = std::env::var("HOME").unwrap();
        Self {
            main: MainConfig {
                datadir: format!("{home}/.baza"),
                box_delimiter: "::".into(),
                bundle_delimiter: ".".into(),
            },
            gitfs: GitFsConfig {
                url: None,
                privatekey: None,
                passphrase: None,
            },
            storage: StorageConfig { r#type: Type::Redb },
        }
    }
}

pub enum MessageType {
    Clean,
    Data,
    Info,
    Warning,
    Error,
}

impl Config {
    pub fn get() -> &'static Config {
        CONFIG.get_or_init(Config::default)
    }

    pub fn build(path: &Path) -> BazaR<()> {
        let config = if path.exists() {
            let config = fs::read_to_string(path).map_err(|e| exn::Exn::new(e.into()))?;
            toml::from_str(&config).map_err(|e| exn::Exn::new(e.into()))?
        } else {
            let config = Config::default();
            let config_str = toml::to_string(&config).map_err(|e| exn::Exn::new(e.into()))?;
            fs::create_dir_all(path.parent().unwrap()).map_err(|e| exn::Exn::new(e.into()))?;
            fs::write(path, config_str).map_err(|e| exn::Exn::new(e.into()))?;
            config
        };

        CONFIG.set(config).unwrap();
        Ok(())
    }
}

pub fn generate(
    length: usize,
    use_special: bool,
    use_numbers: bool,
    use_uppercase: bool,
) -> BazaR<String> {
    let mut charset = "abcdefghijklmnopqrstuvwxyz".to_string();
    if use_special {
        charset.push_str("!@#$%^&*()_+-=[]{}|;':\",./<>?");
    }
    if use_numbers {
        charset.push_str("0123456789");
    }
    if use_uppercase {
        charset.push_str("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    }

    let mut rng = rand::rng();
    Ok((0..length)
        .map(|_| {
            let idx = rng.random_range(0..charset.len());
            charset.chars().nth(idx).unwrap()
        })
        .collect())
}

fn as_hash(str: &str) -> [u8; 32] {
    let mut hasher = sha2::Sha256::new();
    hasher.update(str.as_bytes());
    let result = hasher.finalize();
    result.into()
}

pub(crate) fn key_file() -> String {
    let config = Config::get();
    let datadir = &config.main.datadir;
    format!("{datadir}/key.bin")
}

pub fn lock() -> BazaR<()> {
    fs::remove_file(key_file()).map_err(|e| exn::Exn::new(e.into()))?;
    Ok(())
}

pub fn sync() -> BazaR<()> {
    if let Some(_url) = &Config::get().gitfs.url {
        storage::sync()?;
    } else {
        tracing::info!("Please set url for git remote");
    }
    Ok(())
}

pub fn unlock(passphrase: Option<String>) -> BazaR<()> {
    let passphrase = if let Some(passphrase) = passphrase {
        passphrase
    } else {
        let mut passphrase = String::new();
        m("Enter your password: ", MessageType::Warning);
        io::stdout().flush().map_err(|e| exn::Exn::new(e.into()))?;
        io::stdin()
            .read_line(&mut passphrase)
            .map_err(|e| exn::Exn::new(e.into()))?;

        passphrase
    };
    let datadir = &Config::get().main.datadir;
    let key = as_hash(passphrase.trim());
    fs::create_dir_all(datadir).map_err(|e| exn::Exn::new(e.into()))?;
    let mut file = File::create(key_file()).map_err(|e| exn::Exn::new(e.into()))?;
    file.write_all(&key).map_err(|e| exn::Exn::new(e.into()))?;
    Ok(())
}

#[instrument(skip_all)]
pub(crate) fn key() -> BazaR<Vec<u8>> {
    let data = match fs::read(key_file()) {
        Ok(data) => data,
        Err(_) => {
            exn::bail!(crate::error::Error::Message("Failed to read key".into()));
        }
    };
    Ok(data)
}

pub fn m(msg: &str, r#type: MessageType) {
    let msg = match r#type {
        MessageType::Clean => msg,
        MessageType::Data => &format!("{}", msg.bright_blue()),
        MessageType::Info => &format!("{}", msg.bright_green()),
        MessageType::Warning => &format!("{}", msg.bright_yellow()),
        MessageType::Error => &format!("{}", msg.bright_red()),
    };
    print!("{msg}");
}

// TODO: Make with NamedTmpFolder
/// Cleanup temporary files
pub fn cleanup_tmp_folder() -> BazaR<()> {
    let datadir = &Config::get().main.datadir;
    let tmpdir = format!("{datadir}/tmp");
    if std::fs::remove_dir_all(&tmpdir).is_err() {
        tracing::debug!("Tmp folder already cleaned");
    };
    std::fs::create_dir_all(format!("{datadir}/tmp")).map_err(|e| exn::Exn::new(e.into()))?;
    Ok(())
}

pub fn init(passphrase: Option<String>) -> BazaR<()> {
    // Create common folders
    let datadir = &Config::get().main.datadir;
    fs::create_dir_all(format!("{datadir}/data")).map_err(|e| exn::Exn::new(e.into()))?;
    storage::initialize()?;

    // Initialize the default key
    let passphrase = passphrase.unwrap_or(Uuid::new_v4().hyphenated().to_string());
    tracing::info!("Initializing baza in data directory");
    tracing::warn!(passphrase, "!!! Save this password phrase for future use");

    unlock(Some(passphrase))?;

    // Generate default pgp key
    // This is doesn't using now
    // pgp::generate()?;
    Ok(())
}

pub(crate) fn encrypt_file(path: &PathBuf) -> BazaR<()> {
    let data = fs::read(path).map_err(|e| exn::Exn::new(e.into()))?;
    let encrypted = encrypt_data(&data, &key()?)?;
    let mut file = File::create(path).map_err(|e| exn::Exn::new(e.into()))?;
    file.write_all(&encrypted)
        .map_err(|e| exn::Exn::new(e.into()))?;
    Ok(())
}

pub(crate) fn encrypt_data(plaintext: &[u8], key: &[u8]) -> BazaR<Vec<u8>> {
    let cipher = Aes256Gcm::new(key.into());
    let mut nonce = [0u8; 12];
    rand::rng().fill(&mut nonce);
    let nonce = Nonce::from_slice(&nonce);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| exn::Exn::new(e.into()))?;
    Ok([nonce.as_slice(), &ciphertext].concat())
}

pub(crate) fn decrypt_file(path: &PathBuf) -> BazaR<()> {
    let ciphertext = fs::read(path).map_err(|e| exn::Exn::new(e.into()))?;
    let encrypted = decrypt_data(&ciphertext, &key()?)?;
    let mut file = File::create(path).map_err(|e| exn::Exn::new(e.into()))?;
    file.write_all(&encrypted)
        .map_err(|e| exn::Exn::new(e.into()))?;
    Ok(())
}

#[instrument(skip_all)]
pub(crate) fn decrypt_data(ciphertext: &[u8], key: &[u8]) -> BazaR<Vec<u8>> {
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(&ciphertext[..12]);
    let ciphertext = &ciphertext[12..];
    Ok(cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| exn::Exn::new(e.into()))?)
}
