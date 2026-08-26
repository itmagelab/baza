//! # Baza
//!
//! The core library for crate Baza crate
//!

use crate::prelude::*;
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
#[cfg(not(target_arch = "wasm32"))]
use core::str;
use exn::ResultExt;
use rand::RngExt;
use serde::{Deserialize, Serialize};
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
use std::sync::{Arc, Mutex, OnceLock};
use tracing::instrument;
use uuid::Uuid;

pub mod r#box;
pub mod bundle;
pub mod container;
pub mod dump;
pub mod error;
pub mod prelude;
#[cfg(all(not(target_arch = "wasm32"), feature = "s3"))]
pub mod s3;
pub mod storage;
pub mod totp;
pub mod utils;

pub const SYSTEM_BOX: &str = "__baza__";
pub const SALT_KEY: &str = "__baza__::auth::salt";
pub const TOTP_KEY: &str = "__baza__::auth::totp";
pub const TTL_SECONDS: u64 = 15;
pub const PASSWORD_DEFAULT_LEN: usize = 12;
pub const DEFAULT_AUTHOR: &str = "Baza";
pub const TOTP_UUID_KEY: &str = "__baza__::auth::totp::uuid";
pub static CONFIG: OnceLock<Config> = OnceLock::new();
static SESSION_KEY: OnceLock<Mutex<Option<Vec<u8>>>> = OnceLock::new();
pub type BazaR<T> = Result<T, exn::Exn<error::Error>>;

pub fn is_system_key(key: &str) -> bool {
    let prefix = format!("{}{}", SYSTEM_BOX, Config::get().main.box_delimiter);
    key.starts_with(&prefix)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub main: MainConfig,
    pub storage: StorageConfig,
    #[cfg(feature = "s3")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub s3: Option<S3Config>,
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

#[cfg(feature = "s3")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Config {
    pub endpoint: String,
    pub bucket: String,
    pub region: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_key_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_access_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_style: Option<bool>,
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
            #[cfg(feature = "s3")]
            s3: None,
        }
    }
}

impl Config {
    pub fn get() -> &'static Config {
        CONFIG.get_or_init(Config::default)
    }

    pub fn default_path() -> BazaR<std::path::PathBuf> {
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

pub struct Password {
    pub inner: std::sync::Arc<str>,
}

impl Default for Password {
    fn default() -> Self {
        Password::generate(PASSWORD_DEFAULT_LEN, false, false, false)
    }
}

impl Password {
    pub fn new(s: &str) -> Self {
        Self {
            inner: Arc::from(s),
        }
    }

    pub fn generate(length: usize, no_latters: bool, no_numbers: bool, no_symbols: bool) -> Self {
        let latters = "abcdefghijklmnopqrstuvwxyz\
                         ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let numbers = "0123456789";
        let symbols = "!@#$%^&*()_-+=<>?";

        let mut charset: String = Default::default();

        no_latters.not().then(|| charset.push_str(latters));
        no_numbers.not().then(|| charset.push_str(numbers));
        no_symbols.not().then(|| charset.push_str(symbols));

        let mut rng = rand::rng();
        let password: String = (0..length)
            .map(|_| {
                let idx = rng.random_range(0..charset.len());
                charset.chars().nth(idx).unwrap_or('a')
            })
            .collect();
        Self::new(&password)
    }

    pub fn as_str(&self) -> String {
        self.inner.to_string()
    }
}

pub fn lock() -> BazaR<()> {
    if SESSION_KEY.get().is_some() {
        set_session_key(None)?;
    }
    Ok(())
}

pub fn set_session_key(key: Option<Vec<u8>>) -> BazaR<()> {
    let mutex = SESSION_KEY.get_or_init(|| std::sync::Mutex::new(None));
    let mut guard = mutex
        .lock()
        .map_err(|_| crate::error::Error::Message("Failed to lock key mutex".into()))?;
    *guard = key;
    Ok(())
}

/// Returns the stored KDF salt, if the database has one.
pub async fn get_stored_salt() -> Option<Vec<u8>> {
    storage::with_backend(|backend| backend.get(crate::SALT_KEY))
        .await
        .ok()
}

/// Returns the stored KDF salt, generating and persisting a new one if absent.
async fn ensure_salt() -> BazaR<Vec<u8>> {
    if let Some(salt) = get_stored_salt().await {
        return Ok(salt);
    }
    let mut salt = [0u8; 16];
    rand::rng().fill(&mut salt);
    storage::with_backend(|backend| backend.set(crate::SALT_KEY, salt.to_vec()))
        .await
        .or_raise(|| error::Error::Message("Failed to persist KDF salt".into()))?;
    Ok(salt.to_vec())
}

/// Derives the master key with Argon2id when a salt is stored,
/// falling back to the legacy SHA-256 derivation otherwise.
async fn resolve_key_bytes(passphrase: &str) -> BazaR<Vec<u8>> {
    match get_stored_salt().await {
        Some(salt) => Ok(crate::utils::derive_key_argon2(passphrase, &salt)?.to_vec()),
        None => {
            tracing::warn!(
                "Weak security: salt not found, falling back to SHA256 key derivation. Please migrate your database."
            );
            Ok(as_hash(passphrase).to_vec())
        }
    }
}

pub async fn unlock(passphrase: String, totp_code: Option<String>) -> BazaR<()> {
    let initialized = storage::is_initialized().await?;
    let key_bytes = resolve_key_bytes(passphrase.trim()).await?;

    // Temporarily unlock by setting the SESSION_KEY so we can read the database
    set_session_key(Some(key_bytes))?;

    if !initialized {
        return Ok(());
    }

    // Determine if TOTP is enabled.
    let keys = match storage::with_backend(|backend| backend.list_keys()).await {
        Ok(k) => k,
        Err(e) => {
            let _ = lock();
            return Err(e);
        }
    };

    let has_totp = keys.contains(&TOTP_KEY.to_string());

    if has_totp {
        let secret_res = storage::get_content(TOTP_KEY).await;
        let secret_base32 = match secret_res {
            Ok(s) => s,
            Err(e) => {
                let _ = lock();
                if e.to_string().contains("aes") || e.to_string().contains("decrypt") {
                    exn::bail!(crate::error::Error::Message("Invalid passphrase".into()));
                } else {
                    return Err(e);
                }
            }
        };

        let code = match totp_code {
            Some(c) => c,
            None => {
                let uuid = storage::get_raw(TOTP_UUID_KEY.to_string())
                    .await
                    .unwrap_or_else(|_| "default".to_string());
                let _ = lock();
                exn::bail!(crate::error::Error::Message(format!(
                    "TOTP code required (ID: {})",
                    uuid
                )));
            }
        };

        let is_valid = match totp::verify_code(&secret_base32, &code) {
            Ok(v) => v,
            Err(e) => {
                let _ = lock();
                return Err(e);
            }
        };

        if !is_valid {
            let uuid = storage::get_raw(TOTP_UUID_KEY.to_string())
                .await
                .unwrap_or_else(|_| "default".to_string());
            let _ = lock();
            exn::bail!(crate::error::Error::Message(format!(
                "Invalid TOTP code (ID: {})",
                uuid
            )));
        }
    }

    tracing::debug!("Vault unlocked");
    Ok(())
}

pub(crate) fn key() -> BazaR<Vec<u8>> {
    let mutex = SESSION_KEY.get_or_init(|| std::sync::Mutex::new(None));
    let guard = mutex
        .lock()
        .map_err(|_| crate::error::Error::Message("Failed to lock key mutex".into()))?;

    match &*guard {
        Some(k) => Ok(k.clone()),
        None => exn::bail!(crate::error::Error::Message(
            "Vault is locked. Use '--passphrase' or 'BAZA_PASSPHRASE' env var".into()
        )),
    }
}

#[cfg(target_arch = "wasm32")]
pub fn cleanup_tmp_folder() -> BazaR<()> {
    Ok(())
}

pub async fn migrate(passphrase: String) -> BazaR<()> {
    if !storage::is_initialized().await? {
        exn::bail!(error::Error::Message("Database not initialized".into()));
    }

    // Check if we are already using Argon2
    if get_stored_salt().await.is_some() {
        exn::bail!(error::Error::Message(
            "Database is already migrated to Argon2".into()
        ));
    }

    let passphrase = passphrase.trim();
    let key_bytes_old = as_hash(passphrase).to_vec();
    set_session_key(Some(key_bytes_old.clone()))?;

    let result = migrate_inner(passphrase, &key_bytes_old).await;
    if result.is_err() {
        // Do not leave the vault unlocked with the old key on failure
        let _ = lock();
    }
    result
}

async fn migrate_inner(passphrase: &str, key_bytes_old: &[u8]) -> BazaR<()> {
    let dump_data = storage::dump().await?;

    let mut salt = [0u8; 16];
    rand::rng().fill(&mut salt);
    let key_bytes_new = crate::utils::derive_key_argon2(passphrase, &salt)?.to_vec();

    let mut migrated_data = reencrypt_dump(dump_data, key_bytes_old, &key_bytes_new)?;

    // Add salt to the migrated data
    migrated_data.push((crate::SALT_KEY.to_string(), salt.to_vec()));

    storage::restore(migrated_data).await?;

    set_session_key(Some(key_bytes_new))?;

    Ok(())
}

fn reencrypt_dump(
    dump: Vec<(String, Vec<u8>)>,
    key_old: &[u8],
    key_new: &[u8],
) -> BazaR<Vec<(String, Vec<u8>)>> {
    let mut out = Vec::with_capacity(dump.len() + 1);
    for (key, encrypted_val) in dump {
        if crate::is_system_key(&key) {
            out.push((key, encrypted_val));
            continue;
        }
        let plaintext = crate::decrypt_data(&encrypted_val, key_old).or_raise(|| {
            error::Error::Message(format!(
                "Failed to decrypt key: {} (invalid passphrase?)",
                key
            ))
        })?;
        out.push((key, crate::encrypt_data(&plaintext, key_new)?));
    }
    Ok(out)
}

pub async fn init(passphrase: Option<String>) -> BazaR<String> {
    crate::m("* Initializing Baza database...", crate::MessageType::Info);

    // Create common folders
    #[cfg(not(target_arch = "wasm32"))]
    {
        let datadir = &Config::get().main.datadir;
        crate::m(
            &format!("  [+] Creating directories at: {}", datadir),
            crate::MessageType::Clean,
        );
        fs::create_dir_all(format!("{datadir}/data"))
            .or_raise(|| error::Error::Message("Failed to create data directory".into()))?;
    }

    crate::m(
        "  [+] Initializing database storage...",
        crate::MessageType::Clean,
    );
    storage::initialize()?;

    // Initialize the default key
    crate::m(
        "  [+] Generating master passphrase...",
        crate::MessageType::Clean,
    );
    let passphrase = passphrase.unwrap_or_else(|| Uuid::new_v4().hyphenated().to_string());

    // Make sure a salt is stored before unlocking so that new databases
    // use Argon2 key derivation from the start
    ensure_salt().await?;

    self::unlock(passphrase.clone(), None).await?;

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

pub mod qr;
