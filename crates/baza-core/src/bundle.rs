use crate::{r#box, BazaR, Config};
use core::fmt;
use std::cell::RefCell;
use std::process::Command;
use std::rc::Rc;
use std::sync::Arc;
use tempfile::NamedTempFile;

use self::r#box::BoxRef;

pub(crate) type BundleRef = Rc<RefCell<Bundle>>;

#[derive(Debug)]
pub(crate) struct Bundle {
    pub(crate) name: Arc<str>,
    pub(crate) ptr: Option<Vec<String>>,
    pub(crate) file: NamedTempFile,
    pub(crate) parent: Option<BoxRef>,
}

impl fmt::Display for Bundle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ptr().join(&Config::get().main.box_delimiter))
    }
}

impl Bundle {
    pub(crate) fn new(name: String) -> BazaR<Self> {
        let file = tempfile::Builder::new()
            .tempfile_in(format!("{}/tmp", Config::get().main.datadir))
            .map_err(crate::error::Error::from)?;
        let name = Arc::from(name);
        Ok(Self {
            name,
            file,
            parent: None,
            ptr: None,
        })
    }

    fn ptr(&self) -> Vec<String> {
        let mut pointer = self
            .parent
            .as_ref()
            .map(|parent| parent.borrow().ptr())
            .unwrap_or_default();
        pointer.push(self.name.to_string());

        pointer
    }

    pub(crate) fn create(&self, data: Option<String>) -> BazaR<()> {
        let editor = std::env::var("EDITOR").unwrap_or(String::from("vi"));

        let file = self.file.path().to_path_buf();

        if let Some(str) = data {
            std::fs::write(&file, str).map_err(crate::error::Error::from)?;
        } else {
            let status = Command::new(editor)
                .arg(&file)
                .status()
                .map_err(crate::error::Error::from)?;
            if !status.success() {
                std::process::exit(1);
            }
        };

        crate::encrypt_file(&file)?;
        Ok(())
    }
}
