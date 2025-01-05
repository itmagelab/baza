use std::{cell::RefCell, path::PathBuf, rc::Rc, sync::Arc};

use crate::bundle::Bundle;

#[derive(Debug, Default)]
pub struct r#Box {
    pub name: Arc<str>,
    pub(crate) bundles: Vec<Bundle>,
    pub parent: Option<Rc<RefCell<r#Box>>>,
}

impl r#Box {
    pub(crate) fn new(name: String, parent: Option<Rc<RefCell<r#Box>>>) -> Self {
        let name = Arc::from(name);
        Self {
            name,
            parent,
            ..Default::default()
        }
    }

    pub(crate) fn path(&self) -> PathBuf {
        let mut path = PathBuf::new();
        if let Some(parent) = self.parent.as_ref() {
            path = parent.borrow().path();
        };
        path.push(&*self.name);

        path
    }
}
