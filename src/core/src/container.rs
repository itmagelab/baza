use std::{cell::RefCell, fmt, path::PathBuf, rc::Rc};

use bundle::Bundle;
use io::Read;
use storage::Ctx;

use super::*;

// cover, container, box, bundle
#[derive(Debug, Clone)]
pub struct Container {
    boxes: Vec<Rc<RefCell<r#box::r#Box>>>,
}

impl fmt::Display for Container {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Default, Clone)]
pub struct ContainerBuilder {
    boxes: Vec<Rc<RefCell<r#box::r#Box>>>,
}

impl ContainerBuilder {
    pub fn new() -> Self {
        Self { boxes: vec![] }
    }

    pub fn create_from_str(mut self, name: String) -> BazaR<Self> {
        let mut pack: Vec<&str> = name
            .trim()
            .split(&Config::get().main.box_delimiter)
            .collect();
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

    fn add_bundle(&mut self, mut bundle: Bundle) -> BazaR<&mut Self> {
        if let Some(r#box) = self.boxes.last() {
            bundle.parent = Some(Rc::clone(r#box));
            bundle.ptr = Some(r#box.borrow().ptr());
            r#box.borrow_mut().bundles.push(bundle);
        } else {
            return Err(Error::BoxMoreOne);
        }
        Ok(self)
    }

    pub fn build(self) -> Container {
        let Self { boxes } = self;
        Container { boxes }
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
        name.push(self.bundles().join(&Config::get().main.bundle_delimiter));
        name.join(&Config::get().main.box_delimiter)
    }

    fn create(self, data: Option<String>) -> BazaR<Self> {
        if let Some(r#box) = self.boxes.last() {
            let r#box = r#box.borrow();
            let box_name = r#box.name.to_string();
            let bundle = r#box
                .bundles
                .first()
                .ok_or(Error::BundlesIsEmpty { r#box: box_name })?;
            bundle.create(data)?;
        }
        Ok(self)
    }

    fn save(&mut self, replace: bool) -> BazaR<()> {
        while let Some(r#box) = self.boxes.pop() {
            let mut r#box = r#box.borrow_mut();
            while let Some(bundle) = r#box.bundles.pop() {
                let path = self.ptr(&r#box, &bundle)?;
                bundle.save(path, replace)?;
            }
        }
        Ok(())
    }

    fn edit(self) -> BazaR<Self> {
        if let Some(r#box) = self.boxes.last() {
            let r#box = r#box.borrow();
            let box_name = r#box.name.to_string();
            let bundle = r#box
                .bundles
                .first()
                .ok_or(Error::BundlesIsEmpty { r#box: box_name })?;
            let load_from = self.ptr(&r#box, bundle)?;
            bundle.edit(load_from)?;
        }
        Ok(self)
    }

    fn show(self) -> BazaR<()> {
        if let Some(r#box) = self.boxes.last() {
            let box_name = r#box.borrow().name.to_string();
            let r#box = r#box.borrow();
            let bundle = r#box
                .bundles
                .first()
                .ok_or(Error::BundlesIsEmpty { r#box: box_name })?;
            bundle.show(self.ptr(&r#box, bundle)?)?;
        }
        Ok(())
    }

    fn copy_to_clipboard(self, ttl: u64) -> BazaR<()> {
        if let Some(r#box) = self.boxes.last() {
            let r#box = r#box.borrow();
            let bundle = r#box.bundles.first().ok_or(Error::CommonBazaError)?;
            bundle.copy_to_clipboard(self.ptr(&r#box, bundle)?, ttl)?;
        }
        Ok(())
    }

    fn ptr(&self, r#box: &r#box::r#Box, bundle: &Bundle) -> BazaR<PathBuf> {
        let filename = format!("{}.{}", &*bundle.name, "baza");
        let path = r#box.path().join(filename);
        Ok(path)
    }

    fn delete(&mut self) -> BazaR<()> {
        while let Some(r#box) = self.boxes.pop() {
            let mut r#box = r#box.borrow_mut();
            while let Some(bundle) = r#box.bundles.pop() {
                let path = self.ptr(&r#box, &bundle)?;
                let ctx = if let Some(ptr) = bundle.ptr {
                    let mut fullname = ptr;
                    fullname.push(bundle.name.to_string());
                    let name = fullname.join(&Config::get().main.box_delimiter);
                    Some(Ctx { name })
                } else {
                    None
                };
                storage::delete(path, ctx)?;
            }
        }
        Ok(())
    }
}

pub fn add(str: String) -> BazaR<()> {
    Container::builder()
        .create_from_str(str)?
        .build()
        .create(None)?
        .save(false)?;
    Ok(())
}

pub fn from_stdin(str: String) -> BazaR<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    Container::builder()
        .create_from_str(str)?
        .build()
        .create(Some(input))?
        .save(false)?;
    Ok(())
}

pub fn delete(str: String) -> BazaR<()> {
    Container::builder()
        .create_from_str(str)?
        .build()
        .delete()?;
    Ok(())
}

pub fn edit(str: String) -> BazaR<()> {
    Container::builder()
        .create_from_str(str)?
        .build()
        .edit()?
        .save(true)?;
    Ok(())
}

pub fn copy_to_clipboard(str: String) -> BazaR<()> {
    Container::builder()
        .create_from_str(str)?
        .build()
        .copy_to_clipboard(TTL_SECONDS)?;
    Ok(())
}

pub fn show(str: String) -> BazaR<()> {
    Container::builder().create_from_str(str)?.build().show()?;
    Ok(())
}

pub fn search(str: String) -> BazaR<()> {
    storage::search(str)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let str = "test::my::login".to_string();
        let password = super::generate(255, false, false, false).unwrap();
        init(Some(password.clone())).unwrap();
        unlock(Some(password.clone())).unwrap();
        cleanup_tmp_folder().unwrap();
        Container::builder()
            .create_from_str(str.clone())
            .unwrap()
            .build()
            .delete()
            .unwrap();
        Container::builder()
            .create_from_str(str.clone())
            .unwrap()
            .build()
            .create(Some(password))
            .unwrap()
            .save(false)
            .unwrap();
        Container::builder()
            .create_from_str(str.clone())
            .unwrap()
            .build()
            .show()
            .unwrap();
        Container::builder()
            .create_from_str(str)
            .unwrap()
            .build()
            .delete()
            .unwrap();
        lock().unwrap();
    }
}
