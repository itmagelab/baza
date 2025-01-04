use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Nonce};
use rand::Rng;

fn encrypt_data(plaintext: &[u8], key: &[u8]) -> Vec<u8> {
    let cipher = Aes256Gcm::new(key.into());
    let mut nonce = [0u8; 12];
    rand::thread_rng().fill(&mut nonce);
    let nonce = Nonce::from_slice(&nonce);
    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .expect("Ошибка шифрования!");
    [nonce.as_slice(), &ciphertext].concat()
}

fn decrypt_data(ciphertext: &[u8], key: &[u8]) -> Vec<u8> {
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(&ciphertext[..12]);
    let ciphertext = &ciphertext[12..];
    cipher
        .decrypt(nonce, ciphertext)
        .expect("Ошибка дешифрования!")
}

fn main() {
    let plaintext = b"This is a secret message!";
    let key = rand::thread_rng().gen::<[u8; 32]>();
    let ciphertext = encrypt_data(plaintext, &key);
    let plaintext = decrypt_data(&ciphertext, &key);
    println!("Зашифрованные данные: {:?}", ciphertext);
    println!(
        "Расшифрованные данные: {:?}",
        String::from_utf8_lossy(&plaintext)
    );
}
