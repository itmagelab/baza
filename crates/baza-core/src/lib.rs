//! # Baza
//!
//! The core library for crate Baza crate
//!

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use colored::Colorize;
use core::str;
use exn::ResultExt;
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
    pub storage: StorageConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MainConfig {
    pub datadir: String,
    pub box_delimiter: String,
    pub bundle_delimiter: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageConfig {
    pub r#type: Type,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Type {
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
            let config = fs::read_to_string(path)
                .or_raise(|| error::Error::Message("Failed to read config file".into()))?;
            toml::from_str(&config)
                .or_raise(|| error::Error::Message("Failed to parse config file".into()))?
        } else {
            let config = Config::default();
            let config_str = toml::to_string(&config)
                .or_raise(|| error::Error::Message("Failed to serialize default config".into()))?;
            fs::create_dir_all(path.parent().unwrap())
                .or_raise(|| error::Error::Message("Failed to create config directory".into()))?;
            fs::write(path, config_str)
                .or_raise(|| error::Error::Message("Failed to write config file".into()))?;
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
    fs::remove_file(key_file())
        .or_raise(|| error::Error::Message("Failed to remove key file".into()))?;
    Ok(())
}

pub fn unlock(passphrase: Option<String>) -> BazaR<()> {
    let passphrase = if let Some(passphrase) = passphrase {
        passphrase
    } else {
        let mut passphrase = String::new();
        m("Enter your password: ", MessageType::Warning);
        io::stdout()
            .flush()
            .or_raise(|| error::Error::Message("Failed to flush stdout".into()))?;
        io::stdin()
            .read_line(&mut passphrase)
            .or_raise(|| error::Error::Message("Failed to read passphrase".into()))?;

        passphrase
    };
    let datadir = &Config::get().main.datadir;
    let key = as_hash(passphrase.trim());
    fs::create_dir_all(datadir)
        .or_raise(|| error::Error::Message("Failed to create data directory".into()))?;
    let mut file = File::create(key_file())
        .or_raise(|| error::Error::Message("Failed to create key file".into()))?;
    file.write_all(&key)
        .or_raise(|| error::Error::Message("Failed to write key to file".into()))?;
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
    std::fs::create_dir_all(format!("{datadir}/tmp"))
        .or_raise(|| error::Error::Message("Failed to create tmp directory".into()))?;
    Ok(())
}

pub fn init(passphrase: Option<String>) -> BazaR<()> {
    // Create common folders
    let datadir = &Config::get().main.datadir;
    fs::create_dir_all(format!("{datadir}/data"))
        .or_raise(|| error::Error::Message("Failed to create data directory".into()))?;
    storage::initialize()?;

    // Initialize the default key
    let passphrase = passphrase.unwrap_or(Uuid::new_v4().hyphenated().to_string());
    tracing::info!("Initializing baza in data directory");
    tracing::warn!(passphrase, "!!! Save this password phrase for future use");

    unlock(Some(passphrase))?;

    Ok(())
}

pub(crate) fn encrypt_file(path: &PathBuf) -> BazaR<()> {
    let data = fs::read(path).or_raise(|| {
        error::Error::Message(format!(
            "Failed to read file for encryption: {}",
            path.display()
        ))
    })?;
    let encrypted = encrypt_data(&data, &key()?)?;
    let mut file = File::create(path).or_raise(|| {
        error::Error::Message(format!(
            "Failed to create file for encryption: {}",
            path.display()
        ))
    })?;
    file.write_all(&encrypted).or_raise(|| {
        error::Error::Message(format!(
            "Failed to write encrypted data to: {}",
            path.display()
        ))
    })?;
    Ok(())
}

pub(crate) fn encrypt_data(plaintext: &[u8], key: &[u8]) -> BazaR<Vec<u8>> {
    let cipher = Aes256Gcm::new(key.into());
    let mut nonce = [0u8; 12];
    rand::rng().fill(&mut nonce);
    let nonce = Nonce::from_slice(&nonce);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .or_raise(|| error::Error::Message("Failed to encrypt data".into()))?;
    Ok([nonce.as_slice(), &ciphertext].concat())
}

pub(crate) fn decrypt_file(path: &PathBuf) -> BazaR<()> {
    let ciphertext = fs::read(path).or_raise(|| {
        error::Error::Message(format!(
            "Failed to read file for decryption: {}",
            path.display()
        ))
    })?;
    let encrypted = decrypt_data(&ciphertext, &key()?)?;
    let mut file = File::create(path).or_raise(|| {
        error::Error::Message(format!(
            "Failed to create file for decryption: {}",
            path.display()
        ))
    })?;
    file.write_all(&encrypted).or_raise(|| {
        error::Error::Message(format!(
            "Failed to write decrypted data to: {}",
            path.display()
        ))
    })?;
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
