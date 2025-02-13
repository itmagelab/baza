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
use uuid::Uuid;

use error::Error;
use rand::Rng;

pub(crate) mod r#box;
pub(crate) mod bundle;
pub mod container;
pub mod error;
pub mod git;
pub mod pgp;

const BOX_SEP: &str = "::";
const BUNDLE_SEP: &str = ",";
pub const BAZA_DIR: &str = ".baza";
pub const DEFAULT_EMAIL: &str = "root@baza";
pub const DEFAULT_AUTHOR: &str = "Root Baza";
pub const TTL_SECONDS: u64 = 45;
static CTX: OnceLock<Arc<Config>> = OnceLock::new();

pub enum MessageType {
    Data,
    Info,
    Warning,
    Error,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub main: MainConfig,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MainConfig {
    pub datadir: String,
}

impl Config {
    fn new() -> Config {
        let home = std::env::var("HOME").unwrap();
        Config {
            main: MainConfig {
                datadir: format!("{}/{}", home, String::from(BAZA_DIR)),
            },
        }
    }

    fn init() -> Self {
        let config_str: String = if Path::new("Baza.toml").exists() {
            tracing::debug!("Use config in current folder Baza.toml");
            fs::read_to_string("Baza.toml").expect("Failed to read config file")
        } else {
            let home = std::env::var("HOME").unwrap();
            let config_path = format!("{}/.Baza.toml", home);
            if !Path::new(&config_path).exists() {
                let config = Config::new();
                let toml = toml::to_string(&config).expect("Failed to serialize struct");
                fs::write(&config_path, toml).expect("Failed to write config file");
            };
            fs::read_to_string(config_path).expect("Failed to read config file")
        };
        toml::from_str(&config_str).expect("Failed to parse TOML")
    }

    fn get_or_init() -> Arc<Self> {
        CTX.get_or_init(|| Arc::new(Config::init())).clone()
    }
}

pub type BazaR<T> = Result<T, Error>;

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
            let idx = rand::thread_rng().gen_range(0..chars.len());
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
    let config = Config::get_or_init();
    let datadir = &config.main.datadir;
    format!("{datadir}/key.bin")
}

pub fn lock() -> BazaR<()> {
    fs::remove_file(key_file())?;
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
    let datadir = &Config::get_or_init().main.datadir;
    let key = as_hash(passphrase.trim());
    fs::create_dir_all(datadir)?;
    let mut file = File::create(key_file())?;
    file.write_all(&key)?;
    Ok(())
}

pub(crate) fn key() -> BazaR<Vec<u8>> {
    let data = match fs::read(key_file()) {
        Ok(data) => data,
        Err(e) => {
            m(
                "No key found. Try using the command `baza unlock`\n",
                MessageType::Error,
            );
            return Err(e.into());
        }
    };
    Ok(data)
}

pub fn m(msg: &str, r#type: MessageType) {
    let msg = match r#type {
        MessageType::Data => format!("{}", msg.bright_blue()),
        MessageType::Info => format!("{}", msg.bright_green()),
        MessageType::Warning => format!("{}", msg.bright_yellow()),
        MessageType::Error => format!("{}", msg.bright_red()),
    };
    print!("{}", msg);
}

pub fn init(passphrase: Option<String>) -> BazaR<()> {
    let config = Config::get_or_init();
    let datadir = &config.main.datadir;
    let passphrase = passphrase.unwrap_or(Uuid::new_v4().hyphenated().to_string());
    m("Initializing baza in data directory\n", MessageType::Info);
    m(
        "!!! Save this password phrase for future use\n",
        MessageType::Warning,
    );
    m("PASSWORD: ", MessageType::Info);
    m(&format!("{}\n", passphrase), MessageType::Data);
    let key = as_hash(&passphrase);
    fs::create_dir_all(datadir)?;
    let mut file = File::create(key_file())?;
    file.write_all(&key)?;

    // Generate default pgp key
    pgp::generate()?;
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
    rand::thread_rng().fill(&mut nonce);
    let nonce = Nonce::from_slice(&nonce);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(Error::EncriptionError)?;
    Ok([nonce.as_slice(), &ciphertext].concat())
}

pub(crate) fn decrypt_file(path: &PathBuf) -> BazaR<()> {
    let data = fs::read(path)?;
    let encrypted = decrypt_data(&data, &key()?)?;
    let mut file = File::create(path)?;
    file.write_all(&encrypted)?;
    Ok(())
}

pub(crate) fn decrypt_data(ciphertext: &[u8], key: &[u8]) -> BazaR<Vec<u8>> {
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(&ciphertext[..12]);
    let ciphertext = &ciphertext[12..];
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(Error::EncriptionError)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let password = super::generate(255, false, false, false).unwrap();
        lock().unwrap();
        unlock(Some(password)).unwrap();
    }
}
