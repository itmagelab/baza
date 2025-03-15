use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use crate::bundle::Bundle;

#[derive(Debug, Default)]
pub(crate) struct r#Box {
    pub(crate) name: Arc<str>,
    pub(crate) bundles: Vec<Bundle>,
    pub(crate) parent: Option<Arc<Mutex<r#Box>>>,
}

impl r#Box {
    pub(crate) fn new(name: String, parent: Option<Arc<Mutex<r#Box>>>) -> Self {
        let name = Arc::from(name);
        Self {
            name,
            parent,
            ..Default::default()
        }
    }

    pub(crate) fn pointer(&self) -> Vec<String> {
        let mut pointer = self
            .parent
            .as_ref()
            .map(|parent| parent.lock().unwrap().pointer())
            .unwrap_or_default();
        pointer.push(self.name.to_string());

        pointer
    }

    pub(crate) fn path(&self) -> PathBuf {
        self.pointer().iter().collect()
    }
}
