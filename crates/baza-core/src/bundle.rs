use crate::{r#box, BazaR, Config};
use core::fmt;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use self::r#box::BoxRef;

pub(crate) type BundleRef = Rc<RefCell<Bundle>>;

#[derive(Debug)]
pub(crate) struct Bundle {
    pub(crate) name: Arc<str>,
    #[cfg(target_arch = "wasm32")]
    pub(crate) data: RefCell<Vec<u8>>,
    pub(crate) parent: Option<BoxRef>,
}

impl fmt::Display for Bundle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.ptr().join(&Config::get().main.box_delimiter))
    }
}

impl Bundle {
    pub(crate) fn new(name: String) -> BazaR<Self> {
        Ok(Self {
            name: name.into(),
            #[cfg(target_arch = "wasm32")]
            data: RefCell::new(vec![]),
            parent: None,
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
}
