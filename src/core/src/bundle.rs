use crate::{decrypt_file, encrypt_file, r#box, BazaR};
use std::cell::RefCell;
use std::fs::{self};
use std::rc::Rc;
use std::sync::Arc;
use std::{
    env,
    path::PathBuf,
    process::{exit, Command},
};
use tempfile::NamedTempFile;
use tracing::instrument;

#[derive(Debug)]
pub struct Bundle {
    pub name: Arc<str>,
    pub file: NamedTempFile,
    pub parent: Option<Rc<RefCell<r#box::r#Box>>>,
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

    pub fn path(&self) -> PathBuf {
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

        let temp_file_path = self.file.path().as_os_str();
        let status = Command::new(editor).arg(temp_file_path).status()?;
        if !status.success() {
            exit(1);
        }

        encrypt_file(&self.file.path().to_path_buf())?;

        Ok(self)
    }

    #[instrument]
    pub(crate) fn edit(self, path: PathBuf) -> BazaR<Self> {
        let editor = env::var("EDITOR").unwrap_or(String::from("vi"));

        decrypt_file(&path)?;

        let file = self.file.path().as_os_str();

        fs::copy(path, file)?;
        let status = Command::new(editor).arg(file).status()?;
        if !status.success() {
            exit(1);
        }

        encrypt_file(&self.file.path().to_path_buf())?;

        Ok(self)
    }
}
