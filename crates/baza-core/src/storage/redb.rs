use std::{
    fs::File,
    io::{BufRead, Read},
    process::{exit, Command},
};

use arboard::Clipboard;
use colored::Colorize;
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};

use crate::{
    decrypt_file, encrypt_file, m, storage::StorageBackend, BazaR, Config, MessageType, TTL_SECONDS,
};

use super::Bundle;

const DIR: &str = "redb";

pub struct Redb {
    path: String,
}

impl Redb {
    pub fn instance() -> BazaR<Self> {
        Self::new()
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

impl StorageBackend for Redb {
    fn sync(&self) -> BazaR<()> {
        exn::bail!(crate::error::Error::Message(
            "Sync is not supported for Redb storage".into()
        ))
    }

    fn create(&self, bundle: Bundle, _replace: bool) -> BazaR<()> {
        let ptr = bundle
            .ptr
            .ok_or(crate::error::Error::Message("No pointer found".into()))?;
        let name = ptr.join(&Config::get().main.box_delimiter);

        let file = bundle.file.path().to_path_buf();
        let buf: Vec<u8> = std::fs::read(file).map_err(|e| exn::Exn::new(e.into()))?;

        let db = Redb::new()?.db()?;
        let write_txn = db.begin_write().map_err(|e| exn::Exn::new(e.into()))?;
        {
            let mut table = write_txn
                .open_table(TABLE)
                .map_err(|e| exn::Exn::new(e.into()))?;
            table
                .insert(&*name, buf)
                .map_err(|e| exn::Exn::new(e.into()))?;
        }
        write_txn.commit().map_err(|e| exn::Exn::new(e.into()))?;
        Ok(())
    }

    fn read(&self, bundle: Bundle) -> BazaR<()> {
        let ptr = bundle
            .ptr
            .ok_or(crate::error::Error::Message("No pointer found".into()))?;
        let name = ptr.join(&Config::get().main.box_delimiter);

        let db = Redb::new()?.db()?;
        let read_txn = db.begin_read().map_err(|e| exn::Exn::new(e.into()))?;
        let table = read_txn
            .open_table(TABLE)
            .map_err(|e| exn::Exn::new(e.into()))?;

        let path = bundle.file.path().to_path_buf();

        let data = table
            .get(&*name)
            .map_err(|e| exn::Exn::new(e.into()))?
            .ok_or(crate::error::Error::Message("No such key".into()))?
            .value();
        std::fs::write(&path, &data).map_err(|e| exn::Exn::new(e.into()))?;
        decrypt_file(&path)?;

        let mut file = File::open(path).map_err(|e| exn::Exn::new(e.into()))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| exn::Exn::new(e.into()))?;

        m(&contents, crate::MessageType::Clean);
        Ok(())
    }

    fn update(&self, bundle: Bundle) -> BazaR<()> {
        let editor = std::env::var("EDITOR").unwrap_or(String::from("vi"));

        let ptr = bundle
            .ptr
            .ok_or(crate::error::Error::Message("No pointer found".into()))?;
        let name = ptr.join(&Config::get().main.box_delimiter);

        let db = Redb::new()?.db()?;
        let read_txn = db.begin_read().map_err(|e| exn::Exn::new(e.into()))?;
        let table = read_txn
            .open_table(TABLE)
            .map_err(|e| exn::Exn::new(e.into()))?;

        let path = bundle.file.path().to_path_buf();

        let data = table
            .get(&*name)
            .map_err(|e| exn::Exn::new(e.into()))?
            .ok_or(crate::error::Error::Message("No such key".into()))?
            .value();
        std::fs::write(&path, &data).map_err(|e| exn::Exn::new(e.into()))?;
        decrypt_file(&path)?;
        let status = Command::new(editor)
            .arg(&path)
            .status()
            .map_err(|e| exn::Exn::new(e.into()))?;
        if !status.success() {
            exit(1);
        }
        encrypt_file(&path)?;

        let buf: Vec<u8> = std::fs::read(path).map_err(|e| exn::Exn::new(e.into()))?;
        let write_txn = db.begin_write().map_err(|e| exn::Exn::new(e.into()))?;
        {
            let mut table = write_txn
                .open_table(TABLE)
                .map_err(|e| exn::Exn::new(e.into()))?;
            table
                .insert(&*name, buf)
                .map_err(|e| exn::Exn::new(e.into()))?;
        }
        write_txn.commit().map_err(|e| exn::Exn::new(e.into()))?;
        Ok(())
    }

    fn delete(&self, bundle: Bundle) -> BazaR<()> {
        let ptr = bundle
            .ptr
            .ok_or(crate::error::Error::Message("No pointer found".into()))?;
        let name = ptr.join(&Config::get().main.box_delimiter);

        let db = Redb::new()?.db()?;
        let write_txn = db.begin_write().map_err(|e| exn::Exn::new(e.into()))?;
        {
            let mut table = write_txn
                .open_table(TABLE)
                .map_err(|e| exn::Exn::new(e.into()))?;
            table.remove(&*name).map_err(|e| exn::Exn::new(e.into()))?;
        }
        write_txn.commit().map_err(|e| exn::Exn::new(e.into()))?;
        Ok(())
    }

    fn search(&self, pattern: String) -> BazaR<()> {
        let db = Redb::new()?.db()?;
        let read_txn = db.begin_read().map_err(|e| exn::Exn::new(e.into()))?;
        let table = read_txn
            .open_table(TABLE)
            .map_err(|e| exn::Exn::new(e.into()))?;

        let re = regex::Regex::new(&pattern).map_err(|e| exn::Exn::new(e.into()))?;

        for result in table.iter().map_err(|e| exn::Exn::new(e.into()))? {
            let (key, _): (redb::AccessGuard<&str>, redb::AccessGuard<Vec<u8>>) =
                result.map_err(|e| exn::Exn::new(e.into()))?;
            if re.is_match(key.value()) {
                m(&format!("{}\n", key.value()), MessageType::Clean);
            }
        }

        Ok(())
    }

    fn copy_to_clipboard(&self, bundle: Bundle, ttl: u64) -> BazaR<()> {
        let mut clipboard = Clipboard::new().map_err(|e| exn::Exn::new(e.into()))?;

        let ptr = bundle
            .ptr
            .ok_or(crate::error::Error::Message("No pointer found".into()))?;
        let name = ptr.join(&Config::get().main.box_delimiter);

        let db = Redb::new()?.db()?;
        let read_txn = db.begin_read().map_err(|e| exn::Exn::new(e.into()))?;
        let table = read_txn
            .open_table(TABLE)
            .map_err(|e| exn::Exn::new(e.into()))?;

        let path = bundle.file.path().to_path_buf();

        let data = table
            .get(&*name)
            .map_err(|e| exn::Exn::new(e.into()))?
            .ok_or(crate::error::Error::Message("No such key".into()))?
            .value();
        std::fs::write(&path, &data).map_err(|e| exn::Exn::new(e.into()))?;
        decrypt_file(&path)?;

        let file = File::open(path).map_err(|e| exn::Exn::new(e.into()))?;

        let mut buffer = std::io::BufReader::new(file);
        let mut first_line = String::new();
        buffer
            .read_line(&mut first_line)
            .map_err(|e| exn::Exn::new(e.into()))?;

        let lossy = first_line.trim();
        clipboard
            .set_text(lossy.trim())
            .map_err(|e| exn::Exn::new(e.into()))?;

        let ttl_duration = std::time::Duration::new(ttl, 0);

        let message = format!("Copied to clipboard. Will clear in {TTL_SECONDS} seconds.");
        println!("{}", message.bright_yellow().bold());
        // TODO: This is start after sleep
        // m(&message, crate::MessageType::Data);
        std::thread::sleep(ttl_duration);
        clipboard
            .set_text("".to_string())
            .map_err(|e| exn::Exn::new(e.into()))?;
        Ok(())
    }
}
