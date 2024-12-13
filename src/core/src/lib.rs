use std::ops::Not;

use error::Error;
use rand::Rng;

pub mod r#box;
pub mod bundle;
pub mod container;
pub mod error;

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
