use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use colored::Colorize;
use uuid::Uuid;
use std::ops::Not;

use error::Error;
use rand::Rng;

pub mod r#box;
pub mod bundle;
pub mod container;
pub mod error;
pub mod pgp;

const SEP: &str = "::";

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

pub fn init() {
    let uuid = Uuid::new_v4().hyphenated().to_string();
    println!("{} {}", "Generate a new token for Baza:".bright_green(), uuid.bright_blue());
    let token = uuid.as_bytes();
    println!("{}", "Enter the password".bright_blue());
    println!("{} {:?}", "Password".bright_green(), token);
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

pub fn decrypt_data(ciphertext: &[u8], key: &[u8]) -> BazaR<Vec<u8>> {
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(&ciphertext[..12]);
    let ciphertext = &ciphertext[12..];
    cipher
        .decrypt(nonce, ciphertext)
        .map_err(Error::EncriptionError)
}
