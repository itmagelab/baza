use std::{
    cell::RefCell,
    env, fmt, fs,
    path::{PathBuf, MAIN_SEPARATOR},
    rc::Rc,
};

use bundle::Bundle;
use tracing::instrument;
use walkdir::{DirEntry, WalkDir};

use super::*;

// cover, container, box, bundle
#[derive(Debug)]
struct Container {
    path: PathBuf,
    boxes: Vec<Rc<RefCell<r#box::r#Box>>>,
}

impl fmt::Display for Container {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Default, Clone)]
struct ContainerBuilder {
    path: PathBuf,
    boxes: Vec<Rc<RefCell<r#box::r#Box>>>,
}

// impl std::str::FromStr for ContainerBuilder {
//     type Err = Error;
//
//     fn from_str(name: &str) -> Result<Self, Self::Err> {
//         let mut pack: Vec<&str> = name.trim().split(SEP).collect();
//         let Some(bundle) = pack.pop() else {
//             return Err(Error::TooFewArguments);
//         };
//         pack.reverse();
//         while let Some(r#box) = pack.pop() {
//             let r#box = r#box::r#Box::new(r#box.to_string(), None);
//             self.add_box(r#box);
//         }
//         let bundle = Bundle::new(bundle.to_string())?;
//         self.add_bundle(bundle);
//         Ok(self)
//     }
// }

impl ContainerBuilder {
    fn new() -> Self {
        let home = env::var("BAZA_DIR").unwrap_or(String::from(BAZA_DIR));
        Self {
            path: PathBuf::from(format!("{}/data", home)),
            boxes: vec![],
        }
    }

    // TODO: Use FromStr instead
    fn create_from_str(mut self, name: String) -> BazaR<Self> {
        let mut pack: Vec<&str> = name.trim().split(BOX_SEP).collect();
        let Some(bundle) = pack.pop() else {
            return Err(Error::TooFewArguments);
        };
        pack.reverse();
        while let Some(r#box) = pack.pop() {
            let r#box = r#box::r#Box::new(r#box.to_string(), None);
            self.add_box(r#box);
        }
        let bundle = Bundle::new(bundle.to_string())?;
        self.add_bundle(bundle)?;
        Ok(self)
    }

    fn add_box(&mut self, mut r#box: r#box::r#Box) -> &mut Self {
        r#box.parent = self.boxes.last().map(Rc::clone);
        self.boxes.push(Rc::new(RefCell::new(r#box)));
        self
    }

    #[instrument]
    fn add_bundle(&mut self, mut bundle: Bundle) -> BazaR<&mut Self> {
        if let Some(r#box) = self.boxes.last() {
            bundle.parent = Some(Rc::clone(r#box));
            r#box.borrow_mut().bundles.push(bundle);
        } else {
            return Err(Error::BoxMoreOne);
        }
        Ok(self)
    }

    fn build(self) -> Container {
        let Self { path: dir, boxes } = self;
        Container { path: dir, boxes }
    }
}

impl Container {
    fn builder() -> ContainerBuilder {
        ContainerBuilder::new()
    }

    fn bundles(&self) -> Vec<String> {
        if let Some(r#box) = self.boxes.last() {
            let bundles = &r#box.borrow().bundles;
            if !bundles.is_empty() {
                return bundles.iter().map(|b| b.name.to_string()).collect();
            };
        };
        vec![]
    }

    fn name(&self) -> String {
        let mut name: Vec<String> = self
            .boxes
            .iter()
            .map(|b| b.borrow().name.to_string())
            .collect();
        name.push(self.bundles().join(BUNDLE_SEP));
        name.join(BOX_SEP)
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
            bundle = bundle.edit(self.path.clone())?;
            r#box.borrow_mut().bundles.push(bundle);
        }
        Ok(self)
    }

    fn copy_to_clipboard(self) -> BazaR<()> {
        if let Some(r#box) = self.boxes.last() {
            let bundle = r#box
                .borrow_mut()
                .bundles
                .pop()
                .ok_or(Error::CommonBazaError)?;
            bundle.copy_to_clipboard(self.path)?;
        }
        Ok(())
    }

    fn save(self) -> BazaR<()> {
        let name = self.name();
        if let Some(r#box) = self.boxes.last() {
            let path = self.path.join(r#box.borrow().path());
            fs::create_dir_all(path.clone())?;
            let bundles = &mut r#box.borrow_mut().bundles;

            while let Some(bundle) = bundles.pop() {
                let path = path.join(&*bundle.name);
                let file = bundle.file;
                file.persist_noclobber(path)?;
            }
        }
        let msg = format!("Bundle {} was added", name);
        git::commit(msg)?;
        Ok(())
    }

    fn rewrite(self) -> BazaR<()> {
        let name = self.name();
        if let Some(r#box) = self.boxes.last() {
            let path = self.path.join(r#box.borrow().path());
            fs::create_dir_all(path.clone())?;
            let bundles = &mut r#box.borrow_mut().bundles;

            while let Some(bundle) = bundles.pop() {
                let path = path.join(&*bundle.name);
                let file = bundle.file;
                file.persist(path)?;
            }
        }
        let msg = format!("Bundle {} was changed", name);
        git::commit(msg)?;
        Ok(())
    }

    fn delete(self) -> BazaR<()> {
        let name = self.name();
        if let Some(r#box) = self.boxes.last() {
            let path = self.path.join(r#box.borrow().path());
            fs::create_dir_all(path.clone())?;
            let bundles = &mut r#box.borrow_mut().bundles;

            while let Some(bundle) = bundles.pop() {
                let path = path.join(&*bundle.name);
                if path.exists() {
                    fs::remove_file(&path)?;
                } else {
                    return Err(Error::BundleNotExist(name));
                };
            }
        }
        let msg = format!("Bundle {} was deleted", name);
        git::commit(msg)?;
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
pub fn delete(str: String) -> BazaR<()> {
    Container::builder()
        .create_from_str(str)?
        .build()
        .delete()?;
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
pub fn copy_to_clipboard(str: String) -> BazaR<()> {
    Container::builder()
        .create_from_str(str)?
        .build()
        .copy_to_clipboard()?;
    Ok(())
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

#[tracing::instrument]
pub fn search(str: String) -> BazaR<()> {
    let builder = ContainerBuilder::new();
    let walker = WalkDir::new(&builder.path).into_iter();
    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() {
            let path = path.strip_prefix(&builder.path)?;
            let lossy = path.to_string_lossy().replace(MAIN_SEPARATOR, "::");

            if lossy.contains(&str) {
                let container = builder.clone().create_from_str(lossy)?.build();
                println!("{}", container);
            }
        }
    }
    Ok(())
}
