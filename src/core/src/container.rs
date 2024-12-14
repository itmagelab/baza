use std::{cell::RefCell, fs, path::PathBuf, rc::Rc};

use bundle::Bundle;

use super::*;

// cover, container, box, bundle
#[derive(Debug)]
struct Container {
    dir: PathBuf,
    name: String,
    boxes: Vec<Rc<RefCell<r#box::r#Box>>>,
}

struct ContainerBuilder {
    dir: PathBuf,
    name: String,
    boxes: Vec<Rc<RefCell<r#box::r#Box>>>,
}

impl ContainerBuilder {
    fn new(name: String) -> Self {
        Self {
            dir: PathBuf::from("/var/tmp/baza"),
            name,
            boxes: vec![],
        }
    }

    fn add_box(&mut self, mut r#box: r#box::r#Box) -> &mut Self {
        let parent = self.boxes.last().map(Rc::clone);
        r#box.parent = parent;
        self.boxes.push(Rc::new(RefCell::new(r#box)));
        self
    }

    fn add_bundle(&mut self, bundle: Bundle) -> &mut Self {
        if let Some(r#box) = self.boxes.last() {
            r#box.borrow_mut().bundles.push(bundle);
        }
        self
    }

    fn build(self) -> Container {
        let Self { dir, name, boxes } = self;
        Container { dir, name, boxes }
    }
}

impl Container {
    fn builder(name: String) -> ContainerBuilder {
        ContainerBuilder::new(name)
    }

    fn save(self) -> BazaR<()> {
        if let Some(r#box) = self.boxes.last() {
            let path = self.dir.join(r#box.borrow().path());
            fs::create_dir_all(path.clone())?;
            let bundles = &mut r#box.borrow_mut().bundles;

            while let Some(bundle) = bundles.pop() {
                let path = path.join(&*bundle.name);
                let file = bundle.file;
                file.persist_noclobber(path).map_err(Error::TempBazaError)?;
            }
        }
        Ok(())
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
#[tracing::instrument]
pub fn create(str: String) -> BazaR<()> {
    let mut builder = Container::builder(str.clone());

    let mut pack: Vec<&str> = str.trim().split(SEP).collect();
    let Some(bundle) = pack.pop() else {
        return Err(Error::TooFewArguments);
    };
    pack.reverse();
    while let Some(r#box) = pack.pop() {
        let r#box = r#box::r#Box::new(r#box.to_string(), None);
        builder.add_box(r#box);
    }
    let bundle = Bundle::new(bundle.to_string())?;
    let bundle = bundle.create()?;
    let bundle = bundle.edit()?;
    builder.add_bundle(bundle);
    let container = builder.build();
    container.save()?;
    Ok(())
}
