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
    crate::storage::save_raw(crate::TOTP_UUID_KEY.to_string(), uuid).await?;

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

/// Get the unencrypted TOTP UUID.
pub async fn get_uuid() -> BazaR<String> {
    crate::storage::get_raw(crate::TOTP_UUID_KEY.to_string()).await
}

/// Helper function to construct a TOTP verifier from the stored secret base32 string.
pub fn get_totp(secret_base32: &str) -> BazaR<TOTP> {
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

/// Get the current UNIX timestamp in seconds, compatible with WASM and native platforms.
fn get_timestamp() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        (js_sys::Date::now() / 1000.0) as u64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
    }
}

/// Internal helper to verify the code against the secret base32 string.
pub fn verify_code(secret_base32: &str, code: &str) -> BazaR<bool> {
    let totp = get_totp(secret_base32)?;
    let timestamp = get_timestamp();
    Ok(totp.check(code, timestamp))
}
