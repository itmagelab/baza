use super::StorageBackend;
use crate::BazaR;
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
            if let Some(s) = STORAGE.as_ref() {
                Ok(s)
            } else {
                Err(exn::Exn::new(crate::error::Error::Message(
                    "WebStorage instance unavailable".into(),
                )))
            }
        }
    }
}

#[async_trait(?Send)]
impl StorageBackend for WebStorage {
    async fn is_initialized(&self) -> BazaR<bool> {
        let transaction = self
            .rexie
            .transaction(&[STORE_NAME], TransactionMode::ReadOnly)
            .map_err(|e| crate::error::Error::Message(e.to_string()))?;

        let store = transaction
            .store(STORE_NAME)
            .map_err(|e| crate::error::Error::Message(e.to_string()))?;

        let keys = store
            .get_all_keys(None, Some(1))
            .await
            .map_err(|e| crate::error::Error::Message(e.to_string()))?;

        Ok(!keys.is_empty())
    }

    async fn list_keys(&self) -> BazaR<Vec<String>> {
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
            .map_err(|e| crate::error::Error::Message(e.to_string()))?;

        let mut result = Vec::new();
        for key in keys {
            if let Some(s) = key.as_string() {
                result.push(s);
            }
        }

        Ok(result)
    }

    async fn get(&self, key: &str) -> BazaR<Vec<u8>> {
        let transaction = self
            .rexie
            .transaction(&[STORE_NAME], TransactionMode::ReadOnly)
            .map_err(|e| crate::error::Error::Message(e.to_string()))?;

        let store = transaction
            .store(STORE_NAME)
            .map_err(|e| crate::error::Error::Message(e.to_string()))?;

        let js_key = JsValue::from_str(key);
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

        Ok(data)
    }

    async fn set(&self, key: &str, value: Vec<u8>) -> BazaR<()> {
        let transaction = self
            .rexie
            .transaction(&[STORE_NAME], TransactionMode::ReadWrite)
            .map_err(|e| crate::error::Error::Message(e.to_string()))?;

        let store = transaction
            .store(STORE_NAME)
            .map_err(|e| crate::error::Error::Message(e.to_string()))?;

        let js_value = serde_wasm_bindgen::to_value(&value)
            .map_err(|e| crate::error::Error::Message(e.to_string()))?;
        let js_key = JsValue::from_str(key);

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

    async fn remove(&self, key: &str) -> BazaR<()> {
        let transaction = self
            .rexie
            .transaction(&[STORE_NAME], TransactionMode::ReadWrite)
            .map_err(|e| crate::error::Error::Message(e.to_string()))?;

        let store = transaction
            .store(STORE_NAME)
            .map_err(|e| crate::error::Error::Message(e.to_string()))?;

        let js_key = JsValue::from_str(key);
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
}
