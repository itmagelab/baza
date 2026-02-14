use exn::ResultExt;
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};
use std::sync::OnceLock;

use crate::{storage::StorageBackend, BazaR, Config};

const DIR: &str = "redb";
const TABLE: TableDefinition<&str, Vec<u8>> = TableDefinition::new("passwords");

pub struct Redb {
    path: String,
}

impl Redb {
    pub(crate) fn instance() -> BazaR<&'static dyn StorageBackend> {
        static INSTANCE: OnceLock<Redb> = OnceLock::new();
        Ok(INSTANCE.get_or_init(|| Self::new().expect("Failed to initialize Redb storage")))
    }

    pub fn new() -> BazaR<Self> {
        let folder =
            std::path::PathBuf::from(format!("{}/data/{}", &Config::get().main.datadir, DIR));
        std::fs::create_dir_all(&folder).map_err(|e| exn::Exn::new(e.into()))?;
        let path = format!("{}/db.redb", folder.to_string_lossy());
        Ok(Self { path })
    }

    fn db(&self) -> BazaR<Database> {
        Database::open(&self.path).map_err(|e| exn::Exn::new(e.into()))
    }
}

pub fn initialize() -> BazaR<()> {
    let folder = std::path::PathBuf::from(format!("{}/data/{}", &Config::get().main.datadir, DIR));
    std::fs::create_dir_all(&folder).map_err(|e| exn::Exn::new(e.into()))?;
    let path = format!("{}/db.redb", folder.to_string_lossy());
    let db = Database::create(path).map_err(|e| exn::Exn::new(e.into()))?;
    let write_txn = db.begin_write().or_raise(|| {
        crate::error::Error::Message("Failed to begin write transaction for initialization".into())
    })?;
    {
        let _ = write_txn
            .open_table(TABLE)
            .or_raise(|| crate::error::Error::Message("Failed to create passwords table".into()))?;
    }
    write_txn.commit().or_raise(|| {
        crate::error::Error::Message("Failed to commit initialization transaction".into())
    })?;
    Ok(())
}

use async_trait::async_trait;

#[async_trait(?Send)]
impl StorageBackend for Redb {
    async fn is_initialized(&self) -> BazaR<bool> {
        Ok(std::path::Path::new(&self.path).exists())
    }

    async fn list_keys(&self) -> BazaR<Vec<String>> {
        let db = self.db()?;
        let read_txn = db
            .begin_read()
            .or_raise(|| crate::error::Error::Message("Failed to begin read transaction".into()))?;
        let table = read_txn
            .open_table(TABLE)
            .or_raise(|| crate::error::Error::Message("Failed to open table".into()))?;

        let mut keys = Vec::new();
        for result in table
            .iter()
            .or_raise(|| crate::error::Error::Message("Failed to iterate over table".into()))?
        {
            let (key, _): (redb::AccessGuard<&str>, redb::AccessGuard<Vec<u8>>) =
                result.or_raise(|| {
                    crate::error::Error::Message("Failed to read entry from table".into())
                })?;
            keys.push(key.value().to_string());
        }
        Ok(keys)
    }

    async fn get(&self, key: &str) -> BazaR<Vec<u8>> {
        let db = self.db()?;
        let read_txn = db
            .begin_read()
            .or_raise(|| crate::error::Error::Message("Failed to begin read transaction".into()))?;
        let table = read_txn
            .open_table(TABLE)
            .or_raise(|| crate::error::Error::Message("Failed to open table".into()))?;

        let data = table
            .get(key)
            .or_raise(|| crate::error::Error::Message("Failed to get value from table".into()))?
            .ok_or(crate::error::Error::Message("No such key".into()))?
            .value();
        Ok(data)
    }

    async fn set(&self, key: &str, value: Vec<u8>) -> BazaR<()> {
        let db = self.db()?;
        let write_txn = db.begin_write().or_raise(|| {
            crate::error::Error::Message("Failed to begin write transaction".into())
        })?;
        {
            let mut table = write_txn
                .open_table(TABLE)
                .or_raise(|| crate::error::Error::Message("Failed to open table".into()))?;
            table
                .insert(key, value)
                .or_raise(|| crate::error::Error::Message("Failed to insert into table".into()))?;
        }
        write_txn
            .commit()
            .or_raise(|| crate::error::Error::Message("Failed to commit transaction".into()))?;
        Ok(())
    }

    async fn remove(&self, key: &str) -> BazaR<()> {
        let db = self.db()?;
        let write_txn = db.begin_write().or_raise(|| {
            crate::error::Error::Message("Failed to begin delete transaction".into())
        })?;
        {
            let mut table = write_txn.open_table(TABLE).or_raise(|| {
                crate::error::Error::Message("Failed to open table for deletion".into())
            })?;
            table.remove(key).or_raise(|| {
                crate::error::Error::Message("Failed to remove key from table".into())
            })?;
        }
        write_txn.commit().or_raise(|| {
            crate::error::Error::Message("Failed to commit delete transaction".into())
        })?;
        Ok(())
    }
}
