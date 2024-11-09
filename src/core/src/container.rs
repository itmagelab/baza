use std::{cell::RefCell, path::PathBuf, rc::Rc};

use bundle::Bundle;

use super::*;

// cover, container, box, bundle
#[derive(Debug, Default)]
pub struct Container {
    pub dir: PathBuf,
    pub name: String,
    pub(crate) boxes: Vec<Rc<RefCell<r#box::r#Box>>>,
}

impl Container {
    pub(crate) fn new(name: String) -> Self {
        Self {
            dir: PathBuf::from("/var/tmp/baza"),
            name,
            boxes: vec![],
        }
    }

    pub(crate) fn builder(&mut self) -> &mut Self {
        self
    }

    pub(crate) fn add_box(&mut self, name: String) -> &mut Self {
        let parent = self.boxes.last().map(Rc::clone);
        let r#box = r#box::r#Box::new(name, vec![], parent);
        self.boxes.push(Rc::new(RefCell::new(r#box)));
        self
    }

    pub(crate) fn add_bundle(&mut self, mut bundle: Bundle) -> &mut Self {
        if let Some(r#box) = self.boxes.last_mut() {
            bundle.path = r#box.borrow().path(self.dir.clone());
            bundle.path.push(bundle.name.clone());
            r#box.borrow_mut().bundle.push(bundle)
        }
        self
    }
}

/// ```
/// let container = Container::new()
/// .builder()
/// .add_box("work")
/// .add_bundle("email")
/// .add_bundle("address")
/// .add_box("ldap")
/// .add_bundle("login")
/// .build();
/// ```
pub fn create(str: String) {
    let mut container = Container::new(str.clone());
    let builder = container.builder();

    let mut pack: Vec<&str> = str.trim().split(SEP).collect();
    let bundle = pack.pop().unwrap();
    pack.reverse();
    while let Some(r#box) = pack.pop() {
        builder.add_box(r#box.to_string());
    }
    let bundle = Bundle::new(bundle.to_string());
    builder.add_bundle(bundle);
    println!("{:#?}", container);
}
