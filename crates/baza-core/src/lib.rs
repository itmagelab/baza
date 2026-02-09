//! # Baza
//!
//! The core library for crate Baza crate
//!

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
#[cfg(not(target_arch = "wasm32"))]
use colored::Colorize;
use core::str;
use exn::ResultExt;
use serde::{Deserialize, Serialize};
use sha2::Digest;
#[cfg(target_arch = "wasm32")]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::fs::{self, File};
#[cfg(target_arch = "wasm32")]
use std::io;
#[cfg(not(target_arch = "wasm32"))]
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
    #[serde(rename = "redb", alias = "Redb")]
    Redb,
}

impl Default for Config {
    fn default() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        #[cfg(target_arch = "wasm32")]
        let home = ".".to_string();

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

        let _ = CONFIG.set(config);
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

#[cfg(not(target_arch = "wasm32"))]
pub fn lock() -> BazaR<()> {
    fs::remove_file(key_file())
        .or_raise(|| error::Error::Message("Failed to remove key file".into()))?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
static WASM_KEY: std::sync::OnceLock<std::sync::Mutex<Option<Vec<u8>>>> =
    std::sync::OnceLock::new();

#[cfg(target_arch = "wasm32")]
pub fn lock() -> BazaR<()> {
    if let Some(mutex) = WASM_KEY.get() {
        let mut guard = mutex
            .lock()
            .map_err(|_| crate::error::Error::Message("Failed to lock key mutex".into()))?;
        *guard = None;
    }
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub fn unlock(passphrase: Option<String>) -> BazaR<()> {
    let passphrase = passphrase.ok_or_else(|| {
        crate::error::Error::Message("Passphrase required for WASM unlock".into())
    })?;
    let key_bytes = as_hash(passphrase.trim());

    let mutex = WASM_KEY.get_or_init(|| std::sync::Mutex::new(None));
    let mut guard = mutex
        .lock()
        .map_err(|_| crate::error::Error::Message("Failed to lock key mutex".into()))?;
    *guard = Some(key_bytes.to_vec());

    tracing::info!("WASM Unlock successful");
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn unlock(_passphrase: Option<String>) -> BazaR<()> {
    // Для not WASM unlock ничего не делает, ключ читается с диска
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn key() -> BazaR<Vec<u8>> {
    // Здесь должен быть реальный код получения ключа с диска
    // Для примера: Ok(vec![0; 32])
    Err(exn::Exn::new(error::Error::Message("key() not implemented for native".into())))
}

#[cfg(target_arch = "wasm32")]
pub(crate) fn key() -> BazaR<Vec<u8>> {
    let mutex = WASM_KEY.get_or_init(|| std::sync::Mutex::new(None));
    let guard = mutex
        .lock()
        .map_err(|_| crate::error::Error::Message("Failed to lock key mutex".into()))?;

    match &*guard {
        Some(k) => Ok(k.clone()),
        None => exn::bail!(crate::error::Error::Message(
            "Vault is locked. Use 'unlock <password>'".into()
        )),
    }
}

pub fn m(msg: &str, _type: MessageType) {
    #[cfg(not(target_arch = "wasm32"))]
    let msg = match _type {
        MessageType::Clean => msg,
        MessageType::Data => &format!("{}", msg.bright_blue()),
        MessageType::Info => &format!("{}", msg.bright_green()),
        MessageType::Warning => &format!("{}", msg.bright_yellow()),
        MessageType::Error => &format!("{}", msg.bright_red()),
    };

    #[cfg(target_arch = "wasm32")]
    let msg = msg; // No coloring for WASM log for now

    tracing::info!("{msg}");
}

// TODO: Make with NamedTmpFolder
/// Cleanup temporary files
#[cfg(not(target_arch = "wasm32"))]
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

#[cfg(target_arch = "wasm32")]
pub fn cleanup_tmp_folder() -> BazaR<()> {
    Ok(())
}

pub fn init(passphrase: Option<String>) -> BazaR<()> {
    // Create common folders
    #[cfg(not(target_arch = "wasm32"))]
    {
        let datadir = &Config::get().main.datadir;
        fs::create_dir_all(format!("{datadir}/data"))
            .or_raise(|| error::Error::Message("Failed to create data directory".into()))?;
    }
    storage::initialize()?;

    // Initialize the default key
    let passphrase = passphrase.unwrap_or(Uuid::new_v4().hyphenated().to_string());
    tracing::info!("Initializing baza in data directory");
    tracing::warn!(passphrase, "!!! Save this password phrase for future use");

    self::unlock(Some(passphrase))?;

    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn encrypt_file(path: &PathBuf) -> BazaR<()> {
    let data = fs::read(path).or_raise(|| {
        error::Error::Message(format!(
            "Failed to read file for encryption: {}",
            path.display()
        ))
    })?;
    let encrypted = encrypt_data(&data, &self::key()?)?;
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
    let cipher = Aes256Gcm::new_from_slice(key)
        .or_raise(|| error::Error::Message("Failed to initialize cipher".into()))?;
    let mut nonce_bytes = [0u8; 12];
    rand::rng().fill(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .or_raise(|| error::Error::Message("Failed to encrypt data".into()))?;
    Ok([nonce_bytes.as_slice(), &ciphertext].concat())
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn decrypt_file(path: &PathBuf) -> BazaR<()> {
    let ciphertext = fs::read(path).or_raise(|| {
        error::Error::Message(format!(
            "Failed to read file for decryption: {}",
            path.display()
        ))
    })?;
    let encrypted = decrypt_data(&ciphertext, &self::key()?)?;
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
    let cipher = Aes256Gcm::new_from_slice(key)
        .or_raise(|| error::Error::Message("Failed to initialize cipher".into()))?;
    if ciphertext.len() < 12 {
        exn::bail!(error::Error::Message(
            "Invalid ciphertext: too short".into()
        ));
    }
    let (nonce_bytes, actual_ciphertext) = ciphertext.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);
    Ok(cipher
        .decrypt(nonce, actual_ciphertext)
        .map_err(|e| exn::Exn::new(e.into()))?)
}
