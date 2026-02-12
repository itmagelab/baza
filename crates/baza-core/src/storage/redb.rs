use std::{
    fs::File,
    io::{BufRead, Read},
    process::{exit, Command},
};

use arboard::Clipboard;
use colored::Colorize;
use exn::ResultExt;
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};

use crate::{
    decrypt_file, encrypt_file, m, storage::StorageBackend, BazaR, Config, MessageType, TTL_SECONDS,
};
use std::sync::OnceLock;

use super::Bundle;

const DIR: &str = "redb";

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
        Ok(Database::open(&self.path).map_err(|e| exn::Exn::new(e.into()))?)
    }
}

const TABLE: TableDefinition<&str, Vec<u8>> = TableDefinition::new("passwords");

pub fn initialize() -> BazaR<()> {
    let folder = std::path::PathBuf::from(format!("{}/data/{}", &Config::get().main.datadir, DIR));
    std::fs::create_dir_all(&folder).map_err(|e| exn::Exn::new(e.into()))?;
    let path = format!("{}/db.redb", folder.to_string_lossy());
    Database::create(path).map_err(|e| exn::Exn::new(e.into()))?;

    Ok(())
}

use async_trait::async_trait;

#[async_trait(?Send)]
impl StorageBackend for Redb {
    async fn create(&self, bundle: Bundle, _replace: bool) -> BazaR<()> {
        let ptr = bundle
            .ptr
            .ok_or(crate::error::Error::Message("No pointer found".into()))?;
        let name = ptr.join(&Config::get().main.box_delimiter);

        let file = bundle.file.path().to_path_buf();
        let buf: Vec<u8> = std::fs::read(file)
            .or_raise(|| crate::error::Error::Message("Failed to read bundle file".into()))?;

        let db = self.db()?;
        let write_txn = db.begin_write().or_raise(|| {
            crate::error::Error::Message("Failed to begin write transaction".into())
        })?;
        {
            let mut table = write_txn
                .open_table(TABLE)
                .or_raise(|| crate::error::Error::Message("Failed to open table".into()))?;
            table
                .insert(&*name, buf)
                .or_raise(|| crate::error::Error::Message("Failed to insert into table".into()))?;
        }
        write_txn
            .commit()
            .or_raise(|| crate::error::Error::Message("Failed to commit transaction".into()))?;
        Ok(())
    }

    async fn read(&self, bundle: Bundle) -> BazaR<()> {
        let ptr = bundle
            .ptr
            .ok_or(crate::error::Error::Message("No pointer found".into()))?;
        let name = ptr.join(&Config::get().main.box_delimiter);

        let db = self.db()?;
        let read_txn = db
            .begin_read()
            .or_raise(|| crate::error::Error::Message("Failed to begin read transaction".into()))?;
        let table = read_txn
            .open_table(TABLE)
            .or_raise(|| crate::error::Error::Message("Failed to open table".into()))?;

        let path = bundle.file.path().to_path_buf();

        let data = table
            .get(&*name)
            .or_raise(|| crate::error::Error::Message("Failed to get value from table".into()))?
            .ok_or(crate::error::Error::Message("No such key".into()))?
            .value();
        std::fs::write(&path, &data).or_raise(|| {
            crate::error::Error::Message("Failed to write to temporary file".into())
        })?;
        decrypt_file(&path)?;

        let mut file = File::open(path)
            .or_raise(|| crate::error::Error::Message("Failed to open temporary file".into()))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .or_raise(|| crate::error::Error::Message("Failed to read temporary file".into()))?;

        m(&contents, crate::MessageType::Clean);
        Ok(())
    }

    async fn update(&self, bundle: Bundle) -> BazaR<()> {
        let editor = std::env::var("EDITOR").unwrap_or(String::from("vi"));

        let ptr = bundle
            .ptr
            .ok_or(crate::error::Error::Message("No pointer found".into()))?;
        let name = ptr.join(&Config::get().main.box_delimiter);

        let db = self.db()?;
        let read_txn = db.begin_read().map_err(|e| exn::Exn::new(e.into()))?;
        let table = read_txn
            .open_table(TABLE)
            .map_err(|e| exn::Exn::new(e.into()))?;

        let path = bundle.file.path().to_path_buf();

        let data = table
            .get(&*name)
            .or_raise(|| crate::error::Error::Message("Failed to get value from table".into()))?
            .ok_or(crate::error::Error::Message("No such key".into()))?
            .value();
        std::fs::write(&path, &data).or_raise(|| {
            crate::error::Error::Message("Failed to write to temporary file".into())
        })?;
        decrypt_file(&path)?;
        let status = Command::new(editor)
            .arg(&path)
            .status()
            .or_raise(|| crate::error::Error::Message("Failed to launch editor".into()))?;
        if !status.success() {
            exit(1);
        }
        encrypt_file(&path)?;

        let buf: Vec<u8> = std::fs::read(path)
            .or_raise(|| crate::error::Error::Message("Failed to read edited bundle".into()))?;
        let write_txn = db.begin_write().or_raise(|| {
            crate::error::Error::Message("Failed to begin write transaction".into())
        })?;
        {
            let mut table = write_txn.open_table(TABLE).or_raise(|| {
                crate::error::Error::Message("Failed to open table for update".into())
            })?;
            table.insert(&*name, buf).or_raise(|| {
                crate::error::Error::Message("Failed to insert updated value".into())
            })?;
        }
        write_txn.commit().or_raise(|| {
            crate::error::Error::Message("Failed to commit update transaction".into())
        })?;
        Ok(())
    }

    async fn delete(&self, bundle: Bundle) -> BazaR<()> {
        let name = match bundle.ptr {
            Some(ptr) => ptr.join(&Config::get().main.box_delimiter),
            None => bundle.name.to_string(),
        };

        let db = self.db()?;
        let write_txn = db.begin_write().or_raise(|| {
            crate::error::Error::Message("Failed to begin delete transaction".into())
        })?;
        {
            let mut table = write_txn.open_table(TABLE).or_raise(|| {
                crate::error::Error::Message("Failed to open table for deletion".into())
            })?;
            table.remove(&*name).or_raise(|| {
                crate::error::Error::Message("Failed to remove key from table".into())
            })?;
        }
        write_txn.commit().or_raise(|| {
            crate::error::Error::Message("Failed to commit delete transaction".into())
        })?;
        Ok(())
    }

    async fn search(&self, pattern: String) -> BazaR<()> {
        let db = self.db()?;
        let read_txn = db
            .begin_read()
            .or_raise(|| crate::error::Error::Message("Failed to begin read transaction".into()))?;
        let table = read_txn
            .open_table(TABLE)
            .or_raise(|| crate::error::Error::Message("Failed to open table".into()))?;

        let re = regex_lite::Regex::new(&pattern)
            .or_raise(|| crate::error::Error::Message("Invalid search pattern".into()))?;

        for result in table
            .iter()
            .or_raise(|| crate::error::Error::Message("Failed to iterate over table".into()))?
        {
            let (key, _): (redb::AccessGuard<&str>, redb::AccessGuard<Vec<u8>>) =
                result.or_raise(|| {
                    crate::error::Error::Message("Failed to read entry from table".into())
                })?;
            if re.is_match(key.value()) {
                m(&format!("{}\n", key.value()), MessageType::Clean);
            }
        }

        Ok(())
    }

    async fn copy_to_clipboard(&self, bundle: Bundle, ttl: u64) -> BazaR<()> {
        let mut clipboard = Clipboard::new()
            .or_raise(|| crate::error::Error::Message("Failed to access clipboard".into()))?;

        let ptr = bundle
            .ptr
            .ok_or(crate::error::Error::Message("No pointer found".into()))?;
        let name = ptr.join(&Config::get().main.box_delimiter);

        let db = self.db()?;
        let read_txn = db
            .begin_read()
            .or_raise(|| crate::error::Error::Message("Failed to begin read transaction".into()))?;
        let table = read_txn
            .open_table(TABLE)
            .or_raise(|| crate::error::Error::Message("Failed to open table".into()))?;

        let path = bundle.file.path().to_path_buf();

        let data = table
            .get(&*name)
            .or_raise(|| crate::error::Error::Message("Failed to get value from table".into()))?
            .ok_or(crate::error::Error::Message("No such key".into()))?
            .value();
        std::fs::write(&path, &data).or_raise(|| {
            crate::error::Error::Message("Failed to write to temporary file".into())
        })?;
        decrypt_file(&path)?;

        let file = File::open(path)
            .or_raise(|| crate::error::Error::Message("Failed to open temporary file".into()))?;

        let mut buffer = std::io::BufReader::new(file);
        let mut first_line = String::new();
        buffer
            .read_line(&mut first_line)
            .or_raise(|| crate::error::Error::Message("Failed to read secrets file".into()))?;

        let lossy = first_line.trim();
        clipboard
            .set_text(lossy.trim())
            .or_raise(|| crate::error::Error::Message("Failed to set clipboard text".into()))?;

        let ttl_duration = std::time::Duration::new(ttl, 0);

        let message = format!("Copied to clipboard. Will clear in {TTL_SECONDS} seconds.");
        println!("{}", message.bright_yellow().bold());
        // TODO: This is start after sleep
        // m(&message, crate::MessageType::Data);
        std::thread::sleep(ttl_duration);
        clipboard
            .set_text("".to_string())
            .or_raise(|| crate::error::Error::Message("Failed to clear clipboard".into()))?;
        Ok(())
    }

    async fn is_initialized(&self) -> BazaR<bool> {
        Ok(std::path::Path::new(&self.path).exists())
    }

    async fn get_content(&self, bundle: Bundle) -> BazaR<String> {
        let name = match bundle.ptr {
            Some(ptr) => ptr.join(&Config::get().main.box_delimiter),
            None => bundle.name.to_string(),
        };

        let db = self.db()?;
        let read_txn = db.begin_read().or_raise(|| {
            crate::error::Error::Message("Failed to begin read transaction".into())
        })?;
        let table = read_txn
            .open_table(TABLE)
            .or_raise(|| crate::error::Error::Message("Failed to open table".into()))?;

        let data = table
            .get(&*name)
            .or_raise(|| crate::error::Error::Message("Failed to get value from table".into()))?
            .ok_or(crate::error::Error::Message("No such key".into()))?
            .value();

        let key = crate::key()?;
        let plaintext = crate::decrypt_data(&data, &key)?;
        String::from_utf8(plaintext)
            .map_err(|_| crate::error::Error::Message("Failed to decode utf8".into()).into())
    }

    async fn list_keys(&self) -> BazaR<Vec<String>> {
        let db = self.db()?;
        let read_txn = db.begin_read().or_raise(|| {
            crate::error::Error::Message("Failed to begin read transaction".into())
        })?;
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
}
