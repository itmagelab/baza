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
use std::ops::Not;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use tracing::instrument;
use uuid::Uuid;

use rand::Rng;

pub(crate) mod r#box;
pub(crate) mod bundle;
pub mod container;
pub mod error;
pub mod pgp;
pub mod storage;

const BOX_DELIMITER: &str = "::";
const BUNDLE_DELIMITER: &str = ",";
pub const BAZA_DIR: &str = ".baza";
pub const DEFAULT_EMAIL: &str = "root@baza";
pub const DEFAULT_AUTHOR: &str = "Root Baza";
pub const TTL_SECONDS: u64 = 45;
static CTX: OnceLock<Arc<Config>> = OnceLock::new();

pub enum MessageType {
    Clean,
    Data,
    Info,
    Warning,
    Error,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub main: MainConfig,
    pub gitfs: GitConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MainConfig {
    pub datadir: String,
    pub box_delimiter: String,
    pub bundle_delimiter: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GitConfig {
    pub url: Option<String>,
    pub privatekey: Option<String>,
    pub passphrase: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum r#Type {
    Gitfs,
    Redb,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StorageConfig {
    pub r#type: r#Type,
}

impl Config {
    fn new() -> Config {
        let home = std::env::var("HOME").unwrap();
        Config {
            main: MainConfig {
                box_delimiter: String::from(BOX_DELIMITER),
                bundle_delimiter: String::from(BUNDLE_DELIMITER),
                datadir: format!("{home}/{BAZA_DIR}"),
            },
            gitfs: GitConfig {
                url: None,
                privatekey: None,
                passphrase: None,
            },
            storage: StorageConfig {
                r#type: r#Type::Redb,
            },
        }
    }

    fn init() -> Self {
        let home = std::env::var("HOME").unwrap();
        let config_path = format!("{home}/.config/baza");
        let config_file = format!("{config_path}/baza.toml");
        fs::create_dir_all(&config_path).unwrap();

        let config_str: String = if Path::new(&config_file).exists() {
            fs::read_to_string(&config_file).expect("Failed to read config file")
        } else {
            tracing::info!("A new configuration file has been created");
            let config = Config::new();
            let toml = toml::to_string(&config).expect("Failed to serialize struct");
            fs::write(&config_file, toml).expect("Failed to write config file");
            fs::read_to_string(&config_file).expect("Failed to read config file")
        };
        toml::from_str(&config_str).expect("Failed to parse TOML")
    }

    pub fn get() -> Arc<Self> {
        CTX.get_or_init(|| Arc::new(Config::init())).clone()
    }
}

pub fn generate_config() -> BazaR<()> {
    let config = Config::new();
    let toml = toml::to_string(&config).expect("Failed to serialize struct");
    m(&toml, MessageType::Clean);
    Ok(())
}

pub type BazaR<T> = anyhow::Result<T>;

pub fn generate(length: u8, no_latters: bool, no_symbols: bool, no_numbers: bool) -> BazaR<String> {
    let latters = "abcdefghijklmnopqrstuvwxyz\
                         ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let numbers = "0123456789";
    let symbols = "!@#$%^&*()_-+=<>?";

    let mut chars: String = Default::default();

    no_latters.not().then(|| chars.push_str(latters));
    no_numbers.not().then(|| chars.push_str(numbers));
    no_symbols.not().then(|| chars.push_str(symbols));

    let chars = chars.as_bytes();

    Ok((0..length)
        .map(|_| {
            let idx = rand::rng().random_range(0..chars.len());
            chars[idx] as char
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
    fs::remove_file(key_file())?;
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
        io::stdout().flush()?;
        io::stdin().read_line(&mut passphrase)?;

        passphrase
    };
    let datadir = &Config::get().main.datadir;
    let key = as_hash(passphrase.trim());
    fs::create_dir_all(datadir)?;
    let mut file = File::create(key_file())?;
    file.write_all(&key)?;
    Ok(())
}

#[instrument(skip_all)]
pub(crate) fn key() -> BazaR<Vec<u8>> {
    let data = match fs::read(key_file()) {
        Ok(data) => data,
        Err(_) => {
            anyhow::bail!("Failed to read key");
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
    std::fs::create_dir_all(format!("{datadir}/tmp"))?;
    Ok(())
}

pub fn init(passphrase: Option<String>) -> BazaR<()> {
    // Create common folders
    let datadir = &Config::get().main.datadir;
    fs::create_dir_all(format!("{datadir}/data"))?;
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
    let data = fs::read(path)?;
    let encrypted = encrypt_data(&data, &key()?)?;
    let mut file = File::create(path)?;
    file.write_all(&encrypted)?;
    Ok(())
}

pub(crate) fn encrypt_data(plaintext: &[u8], key: &[u8]) -> BazaR<Vec<u8>> {
    let cipher = Aes256Gcm::new(key.into());
    let mut nonce = [0u8; 12];
    rand::rng().fill(&mut nonce);
    let nonce = Nonce::from_slice(&nonce);
    let ciphertext = cipher.encrypt(nonce, plaintext)?;
    Ok([nonce.as_slice(), &ciphertext].concat())
}

pub(crate) fn decrypt_file(path: &PathBuf) -> BazaR<()> {
    let ciphertext = fs::read(path)?;
    let encrypted = decrypt_data(&ciphertext, &key()?)?;
    let mut file = File::create(path)?;
    file.write_all(&encrypted)?;
    Ok(())
}

#[instrument(skip_all)]
pub(crate) fn decrypt_data(ciphertext: &[u8], key: &[u8]) -> BazaR<Vec<u8>> {
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(&ciphertext[..12]);
    let ciphertext = &ciphertext[12..];
    Ok(cipher.decrypt(nonce, ciphertext)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let password = generate(255, false, false, false);
        assert!(password.is_ok());
        let password = password.unwrap();
        assert!(password.len() == 255);

        assert!(init(Some(password.clone())).is_ok());
        lock().unwrap();
        unlock(Some(password)).unwrap();
    }
}
