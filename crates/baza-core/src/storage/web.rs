use super::StorageBackend;
use crate::bundle::Bundle;
use crate::{BazaR, Config};
use async_trait::async_trait;
use rexie::{Rexie, TransactionMode};
use wasm_bindgen::JsValue;

const DB_NAME: &str = "baza";
const STORE_NAME: &str = "passwords";

static mut STORAGE: Option<WebStorage> = None;

pub struct WebStorage {
    rexie: Rexie,
}

impl WebStorage {
    async fn new() -> BazaR<Self> {
        let rexie = Rexie::builder(DB_NAME)
            .version(1)
            .add_object_store(rexie::ObjectStore::new(STORE_NAME))
            .build()
            .await
            .map_err(|e| crate::error::Error::Message(e.to_string()))?;

        Ok(Self { rexie })
    }

    pub(crate) async fn instance() -> BazaR<&'static Self> {
        unsafe {
            if STORAGE.is_none() {
                STORAGE = Some(Self::new().await?);
            }
            Ok(STORAGE.as_ref().unwrap())
        }
    }
}

#[async_trait(?Send)]
impl StorageBackend for WebStorage {
    async fn create(&self, bundle: Bundle, _replace: bool) -> BazaR<()> {
        let ptr = bundle
            .ptr
            .ok_or(crate::error::Error::Message("No pointer found".into()))?;
        let name = ptr.join(&Config::get().main.box_delimiter);

        // In WASM, bundle.data contains encrypted data (populated by bundle.create)
        let data = bundle.data.borrow().clone();
        if data.is_empty() {
            // If empty, maybe we should not save? Or save empty?
            // Since create() was called, we expect data.
            return Err(crate::error::Error::Message("No data to save".into()).into());
        }

        let transaction = self
            .rexie
            .transaction(&[STORE_NAME], TransactionMode::ReadWrite)
            .map_err(|e| crate::error::Error::Message(e.to_string()))?;

        let store = transaction
            .store(STORE_NAME)
            .map_err(|e| crate::error::Error::Message(e.to_string()))?;

        // Rexie put takes (value, key).
        // We store Vec<u8> as value?
        // IndexedDB supports Uint8Array.
        // serde-wasm-bindgen converts Vec<u8> to array.
        let js_value = serde_wasm_bindgen::to_value(&data)
            .map_err(|e| crate::error::Error::Message(e.to_string()))?;
        let js_key = JsValue::from_str(&name);

        store
            .put(&js_value, Some(&js_key))
            .await
            .map_err(|e| crate::error::Error::Message(e.to_string()))?;

        transaction
            .done()
            .await
            .map_err(|e| crate::error::Error::Message(e.to_string()))?;

        Ok(())
    }

    async fn read(&self, bundle: Bundle) -> BazaR<()> {
        let ptr = bundle
            .ptr
            .ok_or(crate::error::Error::Message("No pointer found".into()))?;
        let name = ptr.join(&Config::get().main.box_delimiter);

        let transaction = self
            .rexie
            .transaction(&[STORE_NAME], TransactionMode::ReadOnly)
            .map_err(|e| crate::error::Error::Message(e.to_string()))?;

        let store = transaction
            .store(STORE_NAME)
            .map_err(|e| crate::error::Error::Message(e.to_string()))?;

        let js_key = JsValue::from_str(&name);
        let js_value = store
            .get(js_key)
            .await
            .map_err(|e| crate::error::Error::Message(e.to_string()))?;

        let js_value = js_value.ok_or(crate::error::Error::Message("No such key".into()))?;

        if js_value.is_null() || js_value.is_undefined() {
            return Err(crate::error::Error::Message("No such key".into()).into());
        }

        let data: Vec<u8> = serde_wasm_bindgen::from_value(js_value)
            .map_err(|e| crate::error::Error::Message(e.to_string()))?;

        let key = crate::key()?; // This relies on key being available (unlocked)
        let plaintext = crate::decrypt_data(&data, &key)?;

        let content = String::from_utf8(plaintext)
            .map_err(|_| crate::error::Error::Message("Failed to decode utf8".into()))?;

        crate::m(&content, crate::MessageType::Clean);

        Ok(())
    }

    async fn update(&self, bundle: Bundle) -> BazaR<()> {
        // Update in WASM essentially means overwrite with new data from bundle.
        // We assume bundle.data has been updated before calling this?
        // But Container logic calls `bundle.update()` which calls `storage::update(bundle)`.
        // In Redb, `storage::update` handles the interactive editor flow.
        // In WASM, since `storage::update` is async but non-interactive (no blocking editor),
        // we might expect `bundle.data` to be populated with NEW encrypted data.
        // However, `bundle.create` populates `data`. `update` flow in `Container` calls `bundle.update()`.
        // If we want to support update, we should implement `create` logic inside `update` or simply alias it?
        // But `update` in `Redb` does decryption -> edit -> encryption.
        // In WASM, the UI likely handles decryption/editing. The `storage::update` should just receive encrypted data.
        // So `WebStorage::update` is same as `create` (put/overwrite).
        self.create(bundle, true).await
    }

    async fn delete(&self, bundle: Bundle) -> BazaR<()> {
        let ptr = bundle
            .ptr
            .ok_or(crate::error::Error::Message("No pointer found".into()))?;
        let name = ptr.join(&Config::get().main.box_delimiter);

        let transaction = self
            .rexie
            .transaction(&[STORE_NAME], TransactionMode::ReadWrite)
            .map_err(|e| crate::error::Error::Message(e.to_string()))?;

        let store = transaction
            .store(STORE_NAME)
            .map_err(|e| crate::error::Error::Message(e.to_string()))?;

        let js_key = JsValue::from_str(&name);
        store
            .delete(js_key)
            .await
            .map_err(|e| crate::error::Error::Message(e.to_string()))?;

        transaction
            .done()
            .await
            .map_err(|e| crate::error::Error::Message(e.to_string()))?;

        Ok(())
    }

    async fn search(&self, pattern: String) -> BazaR<()> {
        let transaction = self
            .rexie
            .transaction(&[STORE_NAME], TransactionMode::ReadOnly)
            .map_err(|e| crate::error::Error::Message(e.to_string()))?;

        let store = transaction
            .store(STORE_NAME)
            .map_err(|e| crate::error::Error::Message(e.to_string()))?;

        let keys = store
            .get_all_keys(None, None)
            .await
            .map_err(|e: rexie::Error| crate::error::Error::Message(e.to_string()))?;

        // getAllKeys returns Result<Indexable, Error> in rexie 0.5?
        // rexie 0.6: getAllKeys returns Result<Vec<JsValue>, Error>?
        // Compiler said `keys.iter()` on `Indexable`? No, compiler said `keys` type inference error?
        // Actually, let's trust that keys is iterable if we fix type inference.
        // Rexie::getAllKeys returns `Result<Vec<JsValue>, Error>` usually.
        // Let's assume Vec<JsValue>.

        let re = regex_lite::Regex::new(&pattern).map_err(crate::error::Error::from)?;

        // iterating on Vec<JsValue>
        for key in keys.iter() {
            if let Some(s) = key.as_string() {
                if re.is_match(&s) {
                    crate::m(&format!("{}\n", s), crate::MessageType::Clean);
                }
            }
        }

        Ok(())
    }

    async fn copy_to_clipboard(&self, bundle: Bundle, ttl: u64) -> BazaR<()> {
        // Read, decrypt, copy.
        let ptr = bundle
            .ptr
            .ok_or(crate::error::Error::Message("No pointer found".into()))?;
        let name = ptr.join(&Config::get().main.box_delimiter);

        let transaction = self
            .rexie
            .transaction(&[STORE_NAME], TransactionMode::ReadOnly)
            .map_err(|e| crate::error::Error::Message(e.to_string()))?;

        let store = transaction
            .store(STORE_NAME)
            .map_err(|e| crate::error::Error::Message(e.to_string()))?;

        let js_key = JsValue::from_str(&name);
        let js_value = store
            .get(js_key)
            .await
            .map_err(|e| crate::error::Error::Message(e.to_string()))?;

        let js_value = js_value.ok_or(crate::error::Error::Message("No such key".into()))?;

        if js_value.is_null() || js_value.is_undefined() {
            return Err(crate::error::Error::Message("No such key".into()).into());
        }

        let data: Vec<u8> = serde_wasm_bindgen::from_value(js_value)
            .map_err(|e| crate::error::Error::Message(e.to_string()))?;

        let key = crate::key()?;
        let plaintext = crate::decrypt_data(&data, &key)?;

        let content = String::from_utf8(plaintext)
            .map_err(|_| crate::error::Error::Message("Failed to decode utf8".into()))?;

        // Take first line? Redb implementation does read_line.
        let first_line = content.lines().next().unwrap_or("").trim();

        // Copy to clipboard using web_sys
        let window = web_sys::window().ok_or(crate::error::Error::Message("No window".into()))?;
        let navigator = window.navigator();
        let clipboard = navigator.clipboard();
        let promise = clipboard.write_text(first_line);
        wasm_bindgen_futures::JsFuture::from(promise)
            .await
            .map_err(|_| crate::error::Error::Message("Failed to write to clipboard".into()))?;

        let message = format!("Copied to clipboard. Will clear in {} seconds.", ttl);
        // crate::m(&message, crate::MessageType::Info); // m() in WASM logs to tracing::info!
        tracing::info!("{}", message);

        // Sleep and clear?
        // In WASM we can't sleep (block). We need setTimeout.
        // But we are in async function.
        // We can spawn a future that waits and clears.
        let ttl_ms = (ttl * 1000) as i32;

        // Use gloo-timers or just web_sys::window().setTimeout
        // We can't easily spawn a detached task here without `wasm_bindgen_futures::spawn_local`.
        // Let's rely on user not needing auto-clear strictly or implement it if possible.
        // A detached future is best.

        wasm_bindgen_futures::spawn_local(async move {
            // Wait
            let promise = js_sys::Promise::new(&mut |resolve, _| {
                web_sys::window()
                    .unwrap()
                    .set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, ttl_ms)
                    .unwrap();
            });
            let _ = wasm_bindgen_futures::JsFuture::from(promise).await;

            // Clear
            if let Some(window) = web_sys::window() {
                let clipboard = window.navigator().clipboard();
                let _ = clipboard.write_text("");
            }
        });

        Ok(())
    }
}
