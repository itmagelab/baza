use std::{cell::RefCell, path::PathBuf, rc::Rc};

use crate::bundle::Bundle;

#[derive(Debug, Clone, Default)]
pub struct r#Box {
    pub name: String,
    pub(crate) bundle: Vec<Bundle>,
    parent: Option<Rc<RefCell<r#Box>>>,
}

impl r#Box {
    pub(crate) fn new(
        name: String,
        bundle: Vec<Bundle>,
        parent: Option<Rc<RefCell<r#Box>>>,
    ) -> Self {
        Self {
            name,
            bundle,
            parent,
        }
    }

    pub fn path(&self, mut path: PathBuf) -> PathBuf {
        if let Some(parent) = self.parent.as_ref() {
            path = parent.borrow().path(path);
        };
        path.push(&self.name);

        path
    }

    pub(crate) fn is_head(&self) -> bool {
        self.parent.is_none()
    }
}
