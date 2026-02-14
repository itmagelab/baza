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
use rand::RngExt;
use serde::{Deserialize, Serialize};
use sha2::Digest;
#[cfg(target_arch = "wasm32")]
use std::fs;
#[cfg(not(target_arch = "wasm32"))]
use std::fs;
#[cfg(target_arch = "wasm32")]
use std::io;
#[cfg(not(target_arch = "wasm32"))]
use std::io;
use std::ops::Not;
use std::path::Path;
use std::sync::OnceLock;
use tracing::instrument;
use uuid::Uuid;

pub mod r#box;
pub mod bundle;
pub mod container;
pub mod dump;
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
        let datadir = if cfg!(debug_assertions) {
            "./.baza".to_string()
        } else {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            format!("{home}/.baza")
        };
        #[cfg(target_arch = "wasm32")]
        let datadir = ".".to_string();

        Self {
            main: MainConfig {
                datadir,
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

    pub fn default_config_path() -> BazaR<std::path::PathBuf> {
        #[cfg(not(target_arch = "wasm32"))]
        if cfg!(debug_assertions) {
            return Ok(std::path::PathBuf::from("./.baza/baza.toml"));
        }

        let home = std::env::var("HOME")
            .or_raise(|| error::Error::Message("Failed to get HOME environment variable".into()))?;

        Ok(std::path::PathBuf::from(format!(
            "{home}/.config/baza/baza.toml"
        )))
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
            let parent = path.parent().ok_or_else(|| {
                exn::Exn::new(error::Error::Message(
                    "Failed to determine config parent directory".into(),
                ))
            })?;
            fs::create_dir_all(parent)
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
    no_latters: bool,
    no_numbers: bool,
    no_symbols: bool,
) -> BazaR<String> {
    let latters = "abcdefghijklmnopqrstuvwxyz\
                         ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let numbers = "0123456789";
    let symbols = "!@#$%^&*()_-+=<>?";

    let mut charset: String = Default::default();

    no_latters.not().then(|| charset.push_str(latters));
    no_numbers.not().then(|| charset.push_str(numbers));
    no_symbols.not().then(|| charset.push_str(symbols));

    let mut rng = rand::rng();
    Ok((0..length)
        .map(|_| {
            let idx = rng.random_range(0..charset.len());
            charset.chars().nth(idx).unwrap_or('a')
        })
        .collect())
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
fn as_hash(str: &str) -> [u8; 32] {
    let mut hasher = sha2::Sha256::new();
    hasher.update(str.as_bytes());
    let result = hasher.finalize();
    result.into()
}

pub(crate) fn key_file() -> String {
    format!("{}/key.txt", &Config::get().main.datadir)
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

    crate::m("WASM Unlock successful", crate::MessageType::Info);
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub fn unlock(passphrase: Option<String>) -> BazaR<()> {
    let passphrase = match passphrase {
        Some(p) => p,
        None => {
            print!("Enter passphrase: ");
            std::io::Write::flush(&mut std::io::stdout()).ok();
            let mut input = String::new();
            std::io::stdin()
                .read_line(&mut input)
                .or_raise(|| error::Error::Message("Failed to read passphrase".into()))?;
            input.trim().to_string()
        }
    };

    fs::write(key_file(), passphrase.trim())
        .or_raise(|| error::Error::Message("Failed to write key file".into()))?;

    crate::m("Vault unlocked", crate::MessageType::Info);
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn key() -> BazaR<Vec<u8>> {
    let path = key_file();
    if !std::path::Path::new(&path).exists() {
        return Err(exn::Exn::new(error::Error::Message(
            "Vault is locked. Use 'unlock' command first".into(),
        )));
    }
    let content = fs::read_to_string(path)
        .or_raise(|| error::Error::Message("Failed to read key file".into()))?;
    Ok(as_hash(content.trim()).to_vec())
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
    {
        let colored_msg = match _type {
            MessageType::Clean => msg.to_string(),
            MessageType::Data => format!("{}", msg.bright_blue()),
            MessageType::Info => format!("{}", msg.bright_green()),
            MessageType::Warning => format!("{}", msg.bright_yellow()),
            MessageType::Error => format!("{}", msg.bright_red()),
        };
        println!("{colored_msg}");
    }

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

pub fn init(passphrase: Option<String>) -> BazaR<String> {
    // Create common folders
    #[cfg(not(target_arch = "wasm32"))]
    {
        let datadir = &Config::get().main.datadir;
        fs::create_dir_all(format!("{datadir}/data"))
            .or_raise(|| error::Error::Message("Failed to create data directory".into()))?;
    }
    storage::initialize()?;

    // Initialize the default key
    let passphrase = passphrase.unwrap_or_else(|| Uuid::new_v4().hyphenated().to_string());
    crate::m(
        "Initializing baza in data directory",
        crate::MessageType::Info,
    );
    crate::m(
        &format!(
            "!!! Save this password phrase for future use: {}",
            passphrase
        ),
        crate::MessageType::Warning,
    );

    self::unlock(Some(passphrase.clone()))?;

    Ok(passphrase)
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
    cipher
        .decrypt(nonce, actual_ciphertext)
        .map_err(|e| exn::Exn::new(e.into()))
}
