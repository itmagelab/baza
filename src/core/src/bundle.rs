use crate::error::Error;
use crate::{decrypt_file, encrypt_file, r#box, BazaR};
use arboard::Clipboard;
use colored::Colorize;
use std::cell::RefCell;
use std::fs::{self};
use std::rc::Rc;
use std::sync::Arc;
use std::{
    env,
    path::PathBuf,
    process::{exit, Command},
};
use std::{thread, time};
use tempfile::NamedTempFile;
use tracing::instrument;

#[derive(Debug)]
pub(crate) struct Bundle {
    pub(crate) name: Arc<str>,
    pub(crate) file: NamedTempFile,
    pub(crate) parent: Option<Rc<RefCell<r#box::r#Box>>>,
}

impl Bundle {
    pub(crate) fn new(name: String) -> BazaR<Self> {
        let file = NamedTempFile::new()?;
        let name = Arc::from(name);
        Ok(Self {
            name,
            file,
            parent: None,
        })
    }

    pub(crate) fn path(&self) -> PathBuf {
        let mut path = self
            .parent
            .as_ref()
            .map(|parent| parent.borrow().path())
            .unwrap_or_default();
        path.push(&*self.name);
        path
    }

    pub(crate) fn create(self) -> BazaR<Self> {
        let editor = env::var("EDITOR").unwrap_or(String::from("vi"));

        let file = self.file.path().to_path_buf();
        let status = Command::new(editor).arg(&file).status()?;
        if !status.success() {
            exit(1);
        }

        encrypt_file(&file)?;

        Ok(self)
    }

    #[instrument]
    pub(crate) fn edit(self, load_from: PathBuf) -> BazaR<Self> {
        let editor = env::var("EDITOR").unwrap_or(String::from("vi"));

        let file = self.file.path().to_path_buf();
        let path = load_from.join(self.path());
        fs::copy(path, &file)?;

        decrypt_file(&file)?;

        let status = Command::new(editor).arg(&file).status()?;
        if !status.success() {
            exit(1);
        }

        encrypt_file(&file)?;

        Ok(self)
    }

    #[instrument]
    pub(crate) fn copy_to_clipboard(self, load_from: PathBuf) -> BazaR<Self> {
        let ttl_seconds = 45;
        let mut clipboard = Clipboard::new().map_err(Error::ArboardError)?;

        let file = self.file.path().to_path_buf();
        let path = load_from.join(self.path());
        fs::copy(path, &file)?;

        decrypt_file(&file)?;
        let data = fs::read(file)?;
        let lossy = String::from_utf8_lossy(&data);
        clipboard
            .set_text(lossy.trim())
            .map_err(Error::ArboardError)?;

        let ttl_duration = time::Duration::new(ttl_seconds, 0);

        let message = "Copied to clipboard. Will clear in 45 seconds.";
        println!("{}", message.bright_yellow().bold());
        thread::sleep(ttl_duration);
        clipboard
            .set_text("".to_string())
            .map_err(Error::ArboardError)?;

        Ok(self)
    }
}
