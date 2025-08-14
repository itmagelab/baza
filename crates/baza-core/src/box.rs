use std::{cell::RefCell, rc::Rc, sync::Arc};

use crate::bundle::BundleRef;

pub type BoxRef = Rc<RefCell<r#Box>>;

#[derive(Debug, Default)]
pub(crate) struct r#Box {
    pub(crate) name: Arc<str>,
    pub(crate) bundles: Vec<BundleRef>,
    pub(crate) parent: Option<BoxRef>,
}

impl r#Box {
    pub(crate) fn new(name: String, parent: Option<BoxRef>) -> Self {
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
