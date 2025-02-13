use crate::error::Error;
use crate::{decrypt_file, encrypt_file, r#box, BazaR, Config, TTL_SECONDS};
use arboard::Clipboard;
use colored::Colorize;
use core::fmt;
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
            self.pointer()
                .join(&Config::get_or_init().main.box_delimiter)
        )
    }
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

    pub(crate) fn pointer(&self) -> Vec<String> {
        let mut pointer = self
            .parent
            .as_ref()
            .map(|parent| parent.borrow().pointer())
            .unwrap_or_default();
        pointer.push(self.name.to_string());

        pointer
    }

    pub(crate) fn path(&self) -> PathBuf {
        self.pointer().iter().collect()
    }

    pub(crate) fn create(self, data: Option<String>) -> BazaR<Self> {
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

        Ok(self)
    }

    #[tracing::instrument(skip_all)]
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

    pub(crate) fn copy_to_clipboard(self, load_from: PathBuf, ttl: u64) -> BazaR<Self> {
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

        let ttl_duration = time::Duration::new(ttl, 0);

        let message = format!(
            "Copied to clipboard. Will clear in {} seconds.",
            TTL_SECONDS
        );
        println!("{}", message.bright_yellow().bold());
        thread::sleep(ttl_duration);
        clipboard
            .set_text("".to_string())
            .map_err(Error::ArboardError)?;

        Ok(self)
    }
}
