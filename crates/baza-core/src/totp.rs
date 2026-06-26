use crate::{storage, BazaR};
use totp_rs::{TOTP, Secret};

pub async fn get_totp_code(name: String) -> BazaR<String> {
    let content = storage::get_content(name).await?;

    let secret_str = extract_totp_secret(&content)
        .ok_or_else(|| crate::error::Error::Message("TOTP secret not found in bundle. Add 'totp: SECRET' or an 'otpauth://' URL.".into()))?;

    let totp = if secret_str.starts_with("otpauth://") {
        TOTP::from_url_unchecked(secret_str)
            .map_err(|e| crate::error::Error::Message(format!("Invalid otpauth URL: {}", e)))?
    } else {
        // Assume it's a raw base32 secret
        let secret = Secret::Encoded(secret_str.to_string())
            .to_bytes()
            .map_err(|e| crate::error::Error::Message(format!("Invalid base32 secret: {}", e)))?;

        TOTP::new_unchecked(
            totp_rs::Algorithm::SHA1,
            6,
            1,
            30,
            secret,
            None,
            "".to_string(),
        )
    };

    Ok(totp.generate_current().map_err(|e| crate::error::Error::Message(format!("Failed to generate TOTP code: {}", e)))?)
}

fn extract_totp_secret(content: &str) -> Option<&str> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("otpauth://") {
            return Some(line);
        }
        if let Some(rest) = line.strip_prefix("totp:") {
            return Some(rest.trim());
        }
    }
    None
}
