use std::{cell::RefCell, fmt, rc::Rc};

use bundle::Bundle;
use io::Read;

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
            let mut ptr = r#box.borrow().ptr();
            ptr.push(bundle.name.to_string());
            bundle.ptr = Some(ptr);
            r#box
                .borrow_mut()
                .bundles
                .push(Rc::new(RefCell::new(bundle)));
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

    fn create(self, data: Option<String>) -> BazaR<Self> {
        if let Some(r#box) = self.boxes.last() {
            let r#box = r#box.borrow();
            let box_name = r#box.name.to_string();
            let bundle = r#box
                .bundles
                .first()
                .ok_or(Error::BundlesIsEmpty { r#box: box_name })?;
            let bundle = bundle.borrow();
            bundle.create(data)?;
        }
        Ok(self)
    }

    fn read(&mut self) -> BazaR<()> {
        if let Some(r#box) = self.boxes.last() {
            let box_name = r#box.borrow().name.to_string();
            let mut r#box = r#box.borrow_mut();
            let bundle = r#box
                .bundles
                .pop()
                .ok_or(Error::BundlesIsEmpty { r#box: box_name })?;
            let bundle = Rc::try_unwrap(bundle)
                .map_err(|_| Error::CommonBazaError)?
                .into_inner();
            storage::read(bundle)?;
        }
        Ok(())
    }

    fn update(self) -> BazaR<Self> {
        if let Some(r#box) = self.boxes.last() {
            let mut r#box = r#box.borrow_mut();
            let box_name = r#box.name.to_string();
            let bundle = r#box
                .bundles
                .pop()
                .ok_or(Error::BundlesIsEmpty { r#box: box_name })?;
            let bundle = Rc::try_unwrap(bundle)
                .map_err(|_| Error::CommonBazaError)?
                .into_inner();
            storage::update(bundle)?;
        }
        Ok(self)
    }

    fn delete(&mut self) -> BazaR<()> {
        while let Some(r#box) = self.boxes.pop() {
            let mut r#box = r#box.borrow_mut();
            while let Some(bundle) = r#box.bundles.pop() {
                let bundle = Rc::try_unwrap(bundle)
                    .map_err(|_| Error::CommonBazaError)?
                    .into_inner();
                storage::delete(bundle)?;
            }
        }
        Ok(())
    }

    fn commit(&mut self) -> BazaR<()> {
        while let Some(r#box) = self.boxes.pop() {
            let mut r#box = r#box.borrow_mut();
            while let Some(bundle) = r#box.bundles.pop() {
                let bundle = Rc::try_unwrap(bundle)
                    .map_err(|_| Error::CommonBazaError)?
                    .into_inner();
                storage::create(bundle)?;
            }
        }
        Ok(())
    }

    fn bundles(&self) -> Vec<String> {
        if let Some(r#box) = self.boxes.last() {
            let bundles = &r#box.borrow().bundles;
            if !bundles.is_empty() {
                return bundles
                    .iter()
                    .map(|b| b.borrow().name.to_string())
                    .collect();
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

    fn copy_to_clipboard(self, ttl: u64) -> BazaR<()> {
        if let Some(r#box) = self.boxes.last() {
            let mut r#box = r#box.borrow_mut();
            let bundle = r#box.bundles.pop().ok_or(Error::CommonBazaError)?;
            // bundle.copy_to_clipboard(self.ptr(&r#box, &bundle)?, ttl)?;
            let bundle = Rc::try_unwrap(bundle)
                .map_err(|_| Error::CommonBazaError)?
                .into_inner();
            storage::copy_to_clipboard(bundle, ttl)?;
        }
        Ok(())
    }
}

pub fn create(str: String) -> BazaR<()> {
    Container::builder()
        .create_from_str(str)?
        .build()
        .create(None)?
        .commit()?;
    Ok(())
}

pub fn read(str: String) -> BazaR<()> {
    Container::builder().create_from_str(str)?.build().read()?;
    Ok(())
}

pub fn update(str: String) -> BazaR<()> {
    Container::builder()
        .create_from_str(str)?
        .build()
        .update()?;
    Ok(())
}

pub fn delete(str: String) -> BazaR<()> {
    Container::builder()
        .create_from_str(str)?
        .build()
        .delete()?;
    Ok(())
}

pub fn copy_to_clipboard(str: String) -> BazaR<()> {
    Container::builder()
        .create_from_str(str)?
        .build()
        .copy_to_clipboard(TTL_SECONDS)?;
    Ok(())
}

pub fn from_stdin(str: String) -> BazaR<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    Container::builder()
        .create_from_str(str)?
        .build()
        .create(Some(input))?
        .commit()?;
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
            .commit()
            .unwrap();
        Container::builder()
            .create_from_str(str.clone())
            .unwrap()
            .build()
            .read()
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
