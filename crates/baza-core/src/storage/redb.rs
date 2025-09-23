use std::{
    fs::File,
    io::{BufRead, Read},
    process::{exit, Command},
};

use arboard::Clipboard;
use colored::Colorize;
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};

use crate::{
    decrypt_file, encrypt_file, m, storage::Storage, BazaR, Config, MessageType, TTL_SECONDS,
};

use super::Bundle;

const DIR: &str = "redb";

pub struct Redb {
    path: String,
}

impl Redb {
    pub fn new() -> BazaR<Self> {
        let folder =
            std::path::PathBuf::from(format!("{}/data/{}", &Config::get().main.datadir, DIR));
        std::fs::create_dir_all(&folder)?;
        let path = format!("{}/db.redb", folder.to_string_lossy());
        Ok(Self { path })
    }

    fn create(&self) -> BazaR<Database> {
        Ok(Database::create(&self.path)?)
    }

    fn db(&self) -> BazaR<Database> {
        Ok(Database::open(&self.path)?)
    }
}

const TABLE: TableDefinition<&str, Vec<u8>> = TableDefinition::new("passwords");

pub fn initialize() -> BazaR<()> {
    Redb::new()?.create()?;

    Ok(())
}

impl Storage for Redb {
    fn create(&self, bundle: Bundle, _replace: bool) -> BazaR<()> {
        let ptr = bundle.ptr.ok_or(anyhow::anyhow!("No pointer found"))?;
        let name = ptr.join(&Config::get().main.box_delimiter);

        let file = bundle.file.path().to_path_buf();
        let buf: Vec<u8> = std::fs::read(file)?;

        let db = Redb::new()?.db()?;
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(TABLE)?;
            table.insert(&*name, buf)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    fn read(&self, bundle: Bundle) -> BazaR<()> {
        let ptr = bundle.ptr.ok_or(anyhow::anyhow!("No pointer found"))?;
        let name = ptr.join(&Config::get().main.box_delimiter);

        let db = Redb::new()?.db()?;
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(TABLE)?;

        let path = bundle.file.path().to_path_buf();

        let data = table
            .get(&*name)?
            .ok_or(anyhow::anyhow!("No such key"))?
            .value();
        std::fs::write(&path, &data)?;
        decrypt_file(&path)?;

        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        m(&contents, crate::MessageType::Clean);
        Ok(())
    }

    fn update(&self, bundle: Bundle) -> BazaR<()> {
        let editor = std::env::var("EDITOR").unwrap_or(String::from("vi"));

        let ptr = bundle.ptr.ok_or(anyhow::anyhow!("No pointer found"))?;
        let name = ptr.join(&Config::get().main.box_delimiter);

        let db = Redb::new()?.db()?;
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(TABLE)?;

        let path = bundle.file.path().to_path_buf();

        let data = table
            .get(&*name)?
            .ok_or(anyhow::anyhow!("No such key"))?
            .value();
        std::fs::write(&path, &data)?;
        decrypt_file(&path)?;
        let status = Command::new(editor).arg(&path).status()?;
        if !status.success() {
            exit(1);
        }
        encrypt_file(&path)?;

        let buf: Vec<u8> = std::fs::read(path)?;
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(TABLE)?;
            table.insert(&*name, buf)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    fn delete(&self, bundle: Bundle) -> BazaR<()> {
        let ptr = bundle.ptr.ok_or(anyhow::anyhow!("No pointer found"))?;
        let name = ptr.join(&Config::get().main.box_delimiter);

        let db = Redb::new()?.db()?;
        let write_txn = db.begin_write()?;
        {
            let mut table = write_txn.open_table(TABLE)?;
            table.remove(&*name)?;
        }
        write_txn.commit()?;
        Ok(())
    }

    fn search(&self, pattern: String) -> BazaR<()> {
        let db = Redb::new()?.db()?;
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(TABLE)?;

        let re = regex::Regex::new(&pattern)?;

        for result in table.iter()? {
            let (key, _) = result?;
            if re.is_match(key.value()) {
                m(&format!("{}\n", key.value()), MessageType::Clean);
            }
        }

        Ok(())
    }

    fn copy_to_clipboard(&self, bundle: Bundle, ttl: u64) -> BazaR<()> {
        let mut clipboard = Clipboard::new()?;

        let ptr = bundle.ptr.ok_or(anyhow::anyhow!("No pointer found"))?;
        let name = ptr.join(&Config::get().main.box_delimiter);

        let db = Redb::new()?.db()?;
        let read_txn = db.begin_read()?;
        let table = read_txn.open_table(TABLE)?;

        let path = bundle.file.path().to_path_buf();

        let data = table
            .get(&*name)?
            .ok_or(anyhow::anyhow!("No such key"))?
            .value();
        std::fs::write(&path, &data)?;
        decrypt_file(&path)?;

        let file = File::open(path)?;

        let mut buffer = std::io::BufReader::new(file);
        let mut first_line = String::new();
        buffer.read_line(&mut first_line)?;

        let lossy = first_line.trim();
        clipboard.set_text(lossy.trim())?;

        let ttl_duration = std::time::Duration::new(ttl, 0);

        let message = format!("Copied to clipboard. Will clear in {TTL_SECONDS} seconds.");
        println!("{}", message.bright_yellow().bold());
        // TODO: This is start after sleep
        // m(&message, crate::MessageType::Data);
        std::thread::sleep(ttl_duration);
        clipboard.set_text("".to_string())?;
        Ok(())
    }
}
