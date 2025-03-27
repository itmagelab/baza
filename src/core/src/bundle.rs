use crate::{r#box, BazaR, Config};
use core::fmt;
use std::cell::RefCell;
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
}
