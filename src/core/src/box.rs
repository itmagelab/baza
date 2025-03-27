use std::{cell::RefCell, rc::Rc, sync::Arc};

use crate::bundle::Bundle;

#[derive(Debug, Default)]
pub(crate) struct r#Box {
    pub(crate) name: Arc<str>,
    pub(crate) bundles: Vec<Rc<RefCell<Bundle>>>,
    pub(crate) parent: Option<Rc<RefCell<r#Box>>>,
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

    pub(crate) fn ptr(&self) -> Vec<String> {
        let mut pointer = self
            .parent
            .as_ref()
            .map(|parent| parent.borrow().ptr())
            .unwrap_or_default();
        pointer.push(self.name.to_string());

        pointer
    }
}
