use crate::{r#box, storage, BazaR, Config};
use core::fmt;
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
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
        tracing::debug!("Creating bundle {:?}", &self.file);
        storage::create(self.file.path().to_path_buf(), data)?;
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    pub(crate) fn edit(&self, load_from: PathBuf) -> BazaR<()> {
        storage::edit(self.file.path().to_path_buf(), load_from)?;
        Ok(())
    }

    pub(crate) fn save(self, path: PathBuf, replace: bool) -> BazaR<()> {
        tracing::debug!("Saving bundle {:?}", &self.file);
        storage::save(self.file, path, replace)?;
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
