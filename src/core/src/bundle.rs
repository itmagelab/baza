use crate::error::Error;
use crate::r#box;
use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;
use std::{
    env,
    fs::{self, File},
    io::Read,
    path::PathBuf,
    process::{exit, Command},
};
use tempfile::NamedTempFile;

#[derive(Debug, Clone, Default)]
pub struct Bundle {
    pub name: String,
    pub parent: Option<Rc<RefCell<r#box::r#Box>>>,
}

impl Bundle {
    pub(crate) fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    fn path(&self) -> PathBuf {
        self.parent
            .as_ref()
            .map(|parent| parent.borrow().path(PathBuf::new()))
            .unwrap_or_default()
    }
}
