use crate::{error::Error, BazaR, TOTP_KEY};
use exn::ResultExt;
use totp_rs::{Algorithm, Secret, TOTP};

/// Generate a new random TOTP secret and register it in the database.
/// Returns the generated secret as base32, the provisioning URI, and the base64 QR code.
pub async fn enable() -> BazaR<(String, String, String)> {
    // Check if vault is unlocked (if not, we cannot get the encryption key to save the secret)
    let _ = crate::key()?;

    // Generate a random secret
    let secret = Secret::generate_secret();
    let secret_base32 = secret.to_encoded().to_string();

    let uuid = uuid::Uuid::new_v4().to_string();

    // Verify it generates a valid URL
    let totp = TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret
            .to_bytes()
            .or_raise(|| Error::Message("Failed to get secret bytes".into()))?,
        Some("Baza".to_string()),
        uuid.clone(),
    )
    .or_raise(|| Error::Message("Failed to initialize TOTP".into()))?;

    let url = totp.get_url();
    let qr_base64 = totp
        .get_qr_base64()
        .map_err(|e| exn::Exn::new(Error::Message(format!("Failed to generate QR code: {}", e))))?;

    // Save the secret and UUID in the database
    crate::storage::save_content(TOTP_KEY.to_string(), secret_base32.clone()).await?;
    crate::storage::save_content(crate::TOTP_UUID_KEY.to_string(), uuid).await?;

    Ok((secret_base32, url, qr_base64))
}

/// Disable TOTP verification by deleting it from the database.
pub async fn disable() -> BazaR<()> {
    // Check if vault is unlocked
    let _ = crate::key()?;

    crate::storage::delete_by_name(TOTP_KEY.to_string()).await?;
    crate::storage::delete_by_name(crate::TOTP_UUID_KEY.to_string()).await
}

/// Check if TOTP is enabled (exists in the database).
pub async fn is_enabled() -> BazaR<bool> {
    let keys = crate::storage::with_backend(|backend| backend.list_keys()).await?;
    Ok(keys.contains(&TOTP_KEY.to_string()))
}

/// Helper function to construct a TOTP verifier from the stored secret base32 string.
fn get_totp(secret_base32: &str) -> BazaR<TOTP> {
    let secret = Secret::Encoded(secret_base32.to_string());
    let secret_bytes = secret
        .to_bytes()
        .or_raise(|| Error::Message("Failed to decode base32 secret".into()))?;

    TOTP::new(
        Algorithm::SHA1,
        6,
        1,
        30,
        secret_bytes,
        Some("Baza".to_string()),
        "Baza".to_string(),
    )
    .or_raise(|| Error::Message("Failed to initialize TOTP".into()))
}

/// Internal helper to verify the code against the secret base32 string.
pub(crate) fn verify_code(secret_base32: &str, code: &str) -> BazaR<bool> {
    let totp = get_totp(secret_base32)?;
    totp.check_current(code)
        .or_raise(|| Error::Message("Failed to check TOTP code due to system time error".into()))
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use super::*;
    use crate::{init, Config};

    #[test]
    fn test_totp_flow() {
        let _lock = crate::TEST_MUTEX.lock().unwrap();
        let test_dir = std::path::PathBuf::from(crate::test_datadir());
        let _ = std::fs::remove_dir_all(&test_dir);
        std::fs::create_dir_all(&test_dir).expect("Failed to create test dir");

        let config_path = test_dir.join("baza.toml");
        let mut config = Config::default();
        config.main.datadir = test_dir.to_string_lossy().to_string();
        let config_str = toml::to_string(&config).expect("Failed to serialize config");
        std::fs::write(&config_path, config_str).expect("Failed to write config");
        Config::build(&config_path).expect("Failed to build config");

        pollster::block_on(async {
            init(Some("test_passphrase".to_string()))
                .await
                .expect("Failed to init database");

            // 1. Check is_enabled initially (should be false)
            assert!(!is_enabled().await.expect("is_enabled failed"));

            // 2. Enable TOTP
            let (secret, url, _qr) = enable().await.expect("enable failed");
            assert!(!secret.is_empty());
            assert!(url.contains("secret="));

            // 3. Check is_enabled again (should be true)
            assert!(is_enabled().await.expect("is_enabled failed"));

            // 4. Verify code
            let totp = get_totp(&secret).expect("get_totp failed");
            let code = totp.generate_current().expect("generate_current failed");
            let valid = verify_code(&secret, &code).expect("verify_code failed");
            assert!(valid);

            // Verify invalid code
            let invalid = verify_code(&secret, "000000").expect("verify_code failed");
            assert!(!invalid);

            // 5. Disable TOTP
            disable().await.expect("disable failed");

            // 6. Check is_enabled (should be false)
            assert!(!is_enabled().await.expect("is_enabled failed"));
        });
    }
}
