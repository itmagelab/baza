use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use colored::Colorize;
use core::str;
use sha2::Digest;
use std::fs::{self, File};
use std::io::Write;
use std::ops::Not;
use std::path::PathBuf;
use uuid::Uuid;

use error::Error;
use rand::Rng;

pub mod r#box;
pub mod bundle;
pub mod container;
pub mod error;
pub mod pgp;

const SEP: &str = "::";
pub const BAZA_DIR: &str = "/var/tmp/baza";

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

pub fn key_file() -> String {
    format!("{BAZA_DIR}/key.bin")
}

pub fn key() -> BazaR<Vec<u8>> {
    let data = fs::read(key_file())?;
    Ok(data)
}

pub fn init(uuid: Option<String>) -> BazaR<()> {
    let uuid = uuid.unwrap_or(Uuid::new_v4().hyphenated().to_string());
    println!("{}", "!!! Save this uuid for future use".bright_yellow());
    println!("{} {}", "Baza:".bright_green(), uuid.bright_blue());
    let key = as_hash(&uuid);
    fs::create_dir_all(BAZA_DIR)?;
    let mut file = File::create(key_file())?;
    file.write_all(&key)?;
    // pgp::generate().unwrap();
    Ok(())
}

pub fn encrypt_file(path: &PathBuf) -> BazaR<()> {
    let data = fs::read(path)?;
    let encrypted = encrypt_data(&data, &key()?)?;
    let mut file = File::create(path)?;
    file.write_all(&encrypted)?;
    Ok(())
}

pub fn encrypt_data(plaintext: &[u8], key: &[u8]) -> BazaR<Vec<u8>> {
    let cipher = Aes256Gcm::new(key.into());
    let mut nonce = [0u8; 12];
    rand::thread_rng().fill(&mut nonce);
    let nonce = Nonce::from_slice(&nonce);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(Error::EncriptionError)?;
    Ok([nonce.as_slice(), &ciphertext].concat())
}

pub fn decrypt_file(path: &PathBuf) -> BazaR<()> {
    let data = fs::read(path)?;
    let encrypted = decrypt_data(&data, &key()?)?;
    let mut file = File::create(path)?;
    file.write_all(&encrypted)?;
    Ok(())
}

pub fn decrypt_data(ciphertext: &[u8], key: &[u8]) -> BazaR<Vec<u8>> {
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(&ciphertext[..12]);
    let ciphertext = &ciphertext[12..];
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(Error::EncriptionError)
}
