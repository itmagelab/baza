use std::{
    cell::RefCell,
    env, fmt, fs,
    path::{PathBuf, MAIN_SEPARATOR},
    rc::Rc,
};

use bundle::Bundle;
use walkdir::WalkDir;

use super::*;

// cover, container, box, bundle
#[derive(Debug)]
struct Container {
    dir: PathBuf,
    name: String,
    boxes: Vec<Rc<RefCell<r#box::r#Box>>>,
}

impl fmt::Display for Container {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _boxes: Vec<String> = self
            .boxes
            .iter()
            .map(|b| b.borrow().name.to_string())
            .collect();
        write!(f, "{}", self.name)
    }
}

#[derive(Debug, Default, Clone)]
struct ContainerBuilder {
    dir: PathBuf,
    name: String,
    boxes: Vec<Rc<RefCell<r#box::r#Box>>>,
}

impl ContainerBuilder {
    fn new() -> Self {
        let home = env::var("BAZA_DIR").unwrap_or(String::from(BAZA_DIR));
        Self {
            dir: PathBuf::from(home),
            boxes: vec![],
            ..Default::default()
        }
    }

    fn create_from_str(mut self, name: String) -> BazaR<Self> {
        self.name = name.clone();
        let mut pack: Vec<&str> = name.trim().split(SEP).collect();
        let Some(bundle) = pack.pop() else {
            return Err(Error::TooFewArguments);
        };
        pack.reverse();
        while let Some(r#box) = pack.pop() {
            let r#box = r#box::r#Box::new(r#box.to_string(), None);
            self.add_box(r#box);
        }
        let bundle = Bundle::new(bundle.to_string())?;
        self.add_bundle(bundle);
        Ok(self)
    }

    fn add_box(&mut self, mut r#box: r#box::r#Box) -> &mut Self {
        let parent = self.boxes.last().map(Rc::clone);
        r#box.parent = parent;
        self.boxes.push(Rc::new(RefCell::new(r#box)));
        self
    }

    fn add_bundle(&mut self, mut bundle: Bundle) -> &mut Self {
        if let Some(r#box) = self.boxes.last() {
            bundle.parent = Some(Rc::clone(r#box));
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
    fn builder() -> ContainerBuilder {
        ContainerBuilder::new()
    }

    fn create(self) -> BazaR<Self> {
        if let Some(r#box) = self.boxes.last() {
            let mut bundle = r#box
                .borrow_mut()
                .bundles
                .pop()
                .ok_or(Error::CommonBazaError)?;
            bundle = bundle.create()?;
            r#box.borrow_mut().bundles.push(bundle);
        }
        Ok(self)
    }

    fn edit(self) -> BazaR<Self> {
        if let Some(r#box) = self.boxes.last() {
            let mut bundle = r#box
                .borrow_mut()
                .bundles
                .pop()
                .ok_or(Error::CommonBazaError)?;
            let path = self.dir.join(bundle.path());
            bundle = bundle.edit(path)?;
            r#box.borrow_mut().bundles.push(bundle);
        }
        Ok(self)
    }

    fn save(self) -> BazaR<()> {
        if let Some(r#box) = self.boxes.last() {
            let path = self.dir.join(r#box.borrow().path());
            fs::create_dir_all(path.clone())?;
            let bundles = &mut r#box.borrow_mut().bundles;

            while let Some(bundle) = bundles.pop() {
                let path = path.join(&*bundle.name);
                let file = bundle.file;
                file.persist_noclobber(path)?;
            }
        }
        Ok(())
    }

    fn rewrite(self) -> BazaR<()> {
        if let Some(r#box) = self.boxes.last() {
            let path = self.dir.join(r#box.borrow().path());
            fs::create_dir_all(path.clone())?;
            let bundles = &mut r#box.borrow_mut().bundles;

            while let Some(bundle) = bundles.pop() {
                let path = path.join(&*bundle.name);
                let file = bundle.file;
                file.persist(path)?;
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
    Container::builder()
        .create_from_str(str)?
        .build()
        .create()?
        .save()?;
    Ok(())
}

#[tracing::instrument]
pub fn edit(str: String) -> BazaR<()> {
    Container::builder()
        .create_from_str(str)?
        .build()
        .edit()?
        .rewrite()?;
    Ok(())
}

#[tracing::instrument]
pub fn search(str: String) -> BazaR<()> {
    let builder = ContainerBuilder::new();
    for entry in WalkDir::new(&builder.dir) {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let path = path.strip_prefix(&builder.dir)?;
            let lossy = path.to_string_lossy().replace(MAIN_SEPARATOR, "::");

            if lossy.contains(&str) {
                let container = builder.clone().create_from_str(lossy)?.build();
                println!("{}", container);
            }
        }
    }
    Ok(())
}
