use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use colored::Colorize;
use core::str;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::fs::{self, File};
use std::io::Write;
use std::ops::Not;
use std::path::{Path, PathBuf};
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
}

pub fn config() -> Config {
    let home = std::env::var("HOME").unwrap();
    let config_path = format!("{}/.Baza.toml", home);
    if !Path::new(&config_path).exists() {
        let config = Config::new();
        let toml = toml::to_string(&config).expect("Failed to serialize struct");
        fs::write(&config_path, toml).expect("Failed to write config file");
    };
    let config_str = fs::read_to_string(config_path).expect("Failed to read config file");
    toml::from_str(&config_str).expect("Failed to parse TOML")
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
    let datadir = config().main.datadir;
    format!("{datadir}/key.bin")
}

pub(crate) fn key() -> BazaR<Vec<u8>> {
    let data = fs::read(key_file())?;
    Ok(data)
}

pub fn init(passphrase: Option<String>) -> BazaR<()> {
    let datadir = config().main.datadir;
    let passphrase = passphrase.unwrap_or(Uuid::new_v4().hyphenated().to_string());
    println!(
        "{}",
        "!!! Save this password phrase for future use".bright_yellow()
    );
    println!(
        "{} {}",
        "Password:".bright_green(),
        passphrase.bright_blue()
    );
    let key = as_hash(&passphrase);
    fs::create_dir_all(&datadir)?;
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
        init(Some(password)).unwrap();
    }
}
