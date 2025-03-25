use crate::{decrypt_file, encrypt_file, m, r#box, storage, BazaR, Config, TTL_SECONDS};
use arboard::Clipboard;
use colored::Colorize;
use core::fmt;
use std::cell::RefCell;
use std::fs::{self, File};
use std::io::{BufRead, Read};
use std::rc::Rc;
use std::sync::Arc;
use std::{
    env,
    path::PathBuf,
    process::{exit, Command},
};
use std::{thread, time};
use tempfile::NamedTempFile;

#[derive(Debug)]
pub(crate) struct Bundle {
    pub(crate) name: Arc<str>,
    pub(crate) file: NamedTempFile,
    pub(crate) parent: Option<Rc<RefCell<r#box::r#Box>>>,
}

impl fmt::Display for Bundle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.pointer().join(&Config::get().main.box_delimiter)
        )
    }
}

impl Bundle {
    pub(crate) fn new(name: String) -> BazaR<Self> {
        let file =
            tempfile::Builder::new().tempfile_in(format!("{}/tmp", Config::get().main.datadir))?;
        let name = Arc::from(name);
        Ok(Self {
            name,
            file,
            parent: None,
        })
    }

    pub(crate) fn pointer(&self) -> Vec<String> {
        let mut pointer = self
            .parent
            .as_ref()
            .map(|parent| parent.borrow().pointer())
            .unwrap_or_default();
        pointer.push(self.name.to_string());

        pointer
    }

    pub(crate) fn create(&self, data: Option<String>) -> BazaR<()> {
        let editor = env::var("EDITOR").unwrap_or(String::from("vi"));

        let file = self.file.path().to_path_buf();
        if let Some(data) = data {
            fs::write(&file, data)?;
        } else {
            let status = Command::new(editor).arg(&file).status()?;
            if !status.success() {
                exit(1);
            }
        };

        encrypt_file(&file)?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub(crate) fn edit(&self, load_from: PathBuf) -> BazaR<()> {
        let editor = env::var("EDITOR").unwrap_or(String::from("vi"));

        let file = self.file.path().to_path_buf();

        fs::copy(load_from, &file)?;

        decrypt_file(&file)?;

        let status = Command::new(editor).arg(&file).status()?;
        if !status.success() {
            exit(1);
        }

        encrypt_file(&file)?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub(crate) fn show(&self, load_from: PathBuf) -> BazaR<()> {
        let filename = self.file.path().to_path_buf();
        fs::copy(load_from, &filename)?;

        decrypt_file(&filename)?;

        let mut file = File::open(filename)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        m(&contents, crate::MessageType::Clean);

        Ok(())
    }

    pub(crate) fn save(self, path: PathBuf, replace: bool) -> BazaR<()> {
        storage::add(self.file, path, replace)?;
        Ok(())
    }

    pub(crate) fn copy_to_clipboard(&self, load_from: PathBuf, ttl: u64) -> BazaR<()> {
        let mut clipboard = Clipboard::new()?;

        let filename = self.file.path().to_path_buf();
        fs::copy(load_from, &filename)?;

        decrypt_file(&filename)?;

        let file = std::fs::File::open(filename)?;
        let mut buffer = std::io::BufReader::new(file);
        let mut first_line = String::new();
        buffer.read_line(&mut first_line)?;

        let lossy = first_line.trim();
        clipboard.set_text(lossy.trim())?;

        let ttl_duration = time::Duration::new(ttl, 0);

        let message = format!(
            "Copied to clipboard. Will clear in {} seconds.",
            TTL_SECONDS
        );
        println!("{}", message.bright_yellow().bold());
        // TODO: This is start after sleep
        // m(&message, crate::MessageType::Data);
        thread::sleep(ttl_duration);
        clipboard.set_text("".to_string())?;

        Ok(())
    }
}
