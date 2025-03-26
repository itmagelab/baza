use crate::storage::Ctx;
use crate::{r#box, storage, BazaR, Config};
use core::fmt;
use std::cell::RefCell;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;
use std::sync::Arc;
use tempfile::NamedTempFile;

#[derive(Debug)]
pub(crate) struct Bundle {
    pub(crate) name: Arc<str>,
    pub(crate) ptr: Option<Vec<String>>,
    pub(crate) file: NamedTempFile,
    pub(crate) parent: Option<Rc<RefCell<r#box::r#Box>>>,
}

impl fmt::Display for Bundle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ptr().join(&Config::get().main.box_delimiter))
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
            ptr: None,
        })
    }

    pub(crate) fn ptr(&self) -> Vec<String> {
        let mut pointer = self
            .parent
            .as_ref()
            .map(|parent| parent.borrow().ptr())
            .unwrap_or_default();
        pointer.push(self.name.to_string());

        pointer
    }

    pub(crate) fn create(&self, data: Option<String>) -> BazaR<()> {
        tracing::debug!("Creating bundle {:?}", &self.file);
        let editor = std::env::var("EDITOR").unwrap_or(String::from("vi"));

        let file = self.file.path().to_path_buf();

        if let Some(str) = data {
            std::fs::write(&file, str)?;
        } else {
            let status = Command::new(editor).arg(&file).status()?;
            if !status.success() {
                std::process::exit(1);
            }
        };

        crate::encrypt_file(&file)?;
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub(crate) fn edit(&self, load_from: PathBuf) -> BazaR<()> {
        let ctx = if let Some(mut ptr) = self.ptr.clone() {
            ptr.push(self.name.to_string());
            let name = ptr.join(&Config::get().main.box_delimiter);
            Some(Ctx { name })
        } else {
            None
        };
        storage::edit(self.file.path().to_path_buf(), load_from, ctx)?;
        Ok(())
    }

    pub(crate) fn save(self, path: PathBuf, _replace: bool) -> BazaR<()> {
        tracing::debug!("Saving bundle {:?}", &self.file);
        let ctx = if let Some(mut ptr) = self.ptr {
            ptr.push(self.name.to_string());
            let name = ptr.join(&Config::get().main.box_delimiter);
            Some(Ctx { name })
        } else {
            None
        };
        let mut file = self.file.reopen()?;
        let mut blob = Vec::new();
        file.read_to_end(&mut blob)?;
        storage::create(&blob, path, ctx)?;
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub(crate) fn show(&self, load_from: PathBuf) -> BazaR<()> {
        storage::show(self.file.path().to_path_buf(), load_from)?;
        Ok(())
    }

    pub(crate) fn copy_to_clipboard(&self, load_from: PathBuf, ttl: u64) -> BazaR<()> {
        storage::copy_to_clipboard(self.file.path().to_path_buf(), load_from, ttl)?;
        Ok(())
    }
}
