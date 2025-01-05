use crate::{decrypt_data, encrypt_data, key, r#box, BazaR};
use std::cell::RefCell;
use std::fs::{self, File};
use std::io::Write;
use std::rc::Rc;
use std::sync::Arc;
use std::{
    env,
    path::PathBuf,
    process::{exit, Command},
};
use tempfile::NamedTempFile;

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
        let data = fs::read(temp_file_path)?;
        let mut file = File::create(temp_file_path)?;
        let encrypted = encrypt_data(&data, &key()?)?;
        file.write_all(&encrypted)?;

        Ok(self)
    }

    pub(crate) fn edit(self, path: PathBuf) -> BazaR<Self> {
        let editor = env::var("EDITOR").unwrap_or(String::from("vi"));

        let data = fs::read(&path)?;
        let mut file = File::create(&path)?;
        let decrypted = decrypt_data(&data, &key()?)?;
        file.write_all(&decrypted)?;

        let file = self.file.path().as_os_str();

        fs::copy(path, file)?;
        let status = Command::new(editor).arg(file).status()?;
        if !status.success() {
            exit(1);
        }

        let data = fs::read(file)?;
        let mut file = File::create(file)?;
        let encrypted = encrypt_data(&data, &key()?)?;
        file.write_all(&encrypted)?;

        Ok(self)
    }
}
