#[cfg(not(target_arch = "wasm32"))]
pub mod redb;
#[cfg(target_arch = "wasm32")]
pub mod web;

#[cfg(not(target_arch = "wasm32"))]
use crate::Config;
use crate::BazaR;
use async_trait::async_trait;

#[cfg(not(target_arch = "wasm32"))]
pub fn storage_dir(dir: &'static str) -> std::path::PathBuf {
    std::path::PathBuf::from(format!("{}/data/{}", &Config::get().main.datadir, dir))
}

#[cfg(not(target_arch = "wasm32"))]
pub(crate) trait StorageBounds: Sync + Send {}
#[cfg(not(target_arch = "wasm32"))]
impl<T: Sync + Send> StorageBounds for T {}

#[cfg(target_arch = "wasm32")]
pub(crate) trait StorageBounds {}
#[cfg(target_arch = "wasm32")]
impl<T> StorageBounds for T {}

#[async_trait(?Send)]
pub(crate) trait StorageBackend: StorageBounds {
    async fn is_initialized(&self) -> BazaR<bool>;
    async fn list_keys(&self) -> BazaR<Vec<String>>;
    async fn get(&self, key: &str) -> BazaR<Vec<u8>>;
    async fn set(&self, key: &str, value: Vec<u8>) -> BazaR<()>;
    async fn remove(&self, key: &str) -> BazaR<()>;
}

pub(crate) async fn with_backend<F, Fut, R>(f: F) -> BazaR<R>
where
    F: FnOnce(&'static dyn StorageBackend) -> Fut,
    Fut: std::future::Future<Output = BazaR<R>>,
{
    #[cfg(not(target_arch = "wasm32"))]
    {
        let backend = self::redb::Redb::instance()?;
        f(backend).await
    }

    #[cfg(target_arch = "wasm32")]
    {
        let backend = self::web::WebStorage::instance().await?;
        f(backend).await
    }
}

pub fn initialize() -> BazaR<()> {
    #[cfg(not(target_arch = "wasm32"))]
    self::redb::initialize()?;
    Ok(())
}

// --- Public Utility Functions (The new "API") ---

pub async fn is_initialized() -> BazaR<bool> {
    with_backend(|backend| backend.is_initialized()).await
}

pub async fn list_all_keys() -> BazaR<Vec<String>> {
    with_backend(|backend| backend.list_keys()).await
}

pub async fn get_content(name: String) -> BazaR<String> {
    let encrypted = with_backend(|backend| backend.get(&name)).await?;
    let key = crate::key()?;
    let plaintext = crate::decrypt_data(&encrypted, &key)?;
    String::from_utf8(plaintext)
        .map_err(|_| crate::error::Error::Message("Failed to decode utf8".into()).into())
}

pub async fn save_content(name: String, content: String) -> BazaR<()> {
    let key = crate::key()?;
    let encrypted = crate::encrypt_data(content.as_bytes(), &key)?;
    with_backend(|backend| backend.set(&name, encrypted)).await
}

pub async fn delete_by_name(name: String) -> BazaR<()> {
    with_backend(|backend| backend.remove(&name)).await
}

pub async fn dump() -> BazaR<Vec<(String, Vec<u8>)>> {
    with_backend(|backend| async move {
        let keys = backend.list_keys().await?;
        let mut data = Vec::with_capacity(keys.len());
        for key in keys {
            let value = backend.get(&key).await?;
            data.push((key, value));
        }
        Ok(data)
    }).await
}

pub async fn restore(data: Vec<(String, Vec<u8>)>) -> BazaR<()> {
    with_backend(|backend| async move {
        // Clear existing data
        let keys = backend.list_keys().await?;
        for key in keys {
            backend.remove(&key).await?;
        }
        
        // Restore new data
        for (key, value) in data {
            backend.set(&key, value).await?;
        }
        Ok(())
    }).await
}

// storage.rs

pub async fn search(pattern: String) -> BazaR<()> {
    let keys = list_all_keys().await?;
    let re = regex_lite::Regex::new(&pattern)
        .map_err(|e| crate::error::Error::Message(e.to_string()))?;
    
    for key in keys {
        if re.is_match(&key) {
            #[cfg(not(target_arch = "wasm32"))]
            crate::m(&key, crate::MessageType::Clean);
            #[cfg(target_arch = "wasm32")]
            tracing::info!("Match: {}", key);
        }
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn copy_to_clipboard(name: String, ttl: u64) -> BazaR<()> {
    use arboard::Clipboard;
    use colored::Colorize;

    let content = get_content(name).await?;
    let first_line = content.lines().next().unwrap_or("").trim();
    
    let mut clipboard = Clipboard::new()
        .map_err(|e| crate::error::Error::Message(e.to_string()))?;
    clipboard.set_text(first_line.to_string())
        .map_err(|e| crate::error::Error::Message(e.to_string()))?;

    println!("{}", format!("Copied to clipboard. Will clear in {} seconds.", ttl).bright_yellow().bold());
    
    std::thread::sleep(std::time::Duration::from_secs(ttl));
    let _ = clipboard.set_text("".to_string());
    Ok(())
}

#[cfg(target_arch = "wasm32")]
pub async fn copy_to_clipboard(_name: String, _ttl: u64) -> BazaR<()> {
    // In WASM, clipboard management is usually handled by the UI (web-sys)
    // because of security restrictions (must be triggered by user gesture).
    // So we just return the content or an error.
    Err(crate::error::Error::Message("Use browser APIs directly for clipboard".into()).into())
}
