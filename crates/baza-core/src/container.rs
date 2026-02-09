use std::{cell::RefCell, fmt, rc::Rc};

use bundle::Bundle;
use exn::ResultExt;
use io::Read;
use r#box::BoxRef;

use super::*;

// cover, container, box, bundle
#[derive(Debug, Clone)]
pub struct Container {
    boxes: Vec<BoxRef>,
}

impl fmt::Display for Container {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Default, Clone)]
pub struct ContainerBuilder {
    boxes: Vec<BoxRef>,
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
            exn::bail!(crate::error::Error::Message(
                "Failed to parse container name".into()
            ));
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
            exn::bail!(crate::error::Error::Message(
                "Failed to add bundle to empty container".into()
            ));
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
            let bundle = r#box.bundles.first().ok_or_else(|| {
                crate::error::Error::Message(format!("The box {box_name} have not bundles"))
            })?;
            let bundle = bundle.borrow();
            bundle.create(data)?;
        }
        Ok(self)
    }

    fn read(&mut self) -> BazaR<()> {
        if let Some(r#box) = self.boxes.last() {
            let box_name = r#box.borrow().name.to_string();
            let mut r#box = r#box.borrow_mut();
            let bundle = r#box.bundles.pop().ok_or_else(|| {
                crate::error::Error::Message(format!("The box {box_name} have not bundles"))
            })?;
            let bundle = Rc::try_unwrap(bundle)
                .map_err(|_| crate::error::Error::Message("Bundle still has references".into()))?
                .into_inner();
            storage::read(bundle)?;
        }
        Ok(())
    }

    fn update(self) -> BazaR<Self> {
        if let Some(r#box) = self.boxes.last() {
            let mut r#box = r#box.borrow_mut();
            let box_name = r#box.name.to_string();
            let bundle = r#box.bundles.pop().ok_or_else(|| {
                crate::error::Error::Message(format!("The box {box_name} have not bundles"))
            })?;
            let bundle = Rc::try_unwrap(bundle)
                .map_err(|_| crate::error::Error::Message("Bundle still has references".into()))?
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
                    .map_err(|_| {
                        crate::error::Error::Message("Bundle still has references".into())
                    })?
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
                    .map_err(|_| {
                        crate::error::Error::Message("Bundle still has references".into())
                    })?
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
            let bundle = r#box
                .bundles
                .pop()
                .ok_or_else(|| crate::error::Error::Message("The box have not bundles".into()))?;
            // bundle.copy_to_clipboard(self.ptr(&r#box, &bundle)?, ttl)?;
            let bundle = Rc::try_unwrap(bundle)
                .map_err(|_| crate::error::Error::Message("Bundle still has references".into()))?
                .into_inner();
            storage::copy_to_clipboard(bundle, ttl)?;
        }
        Ok(())
    }
}

pub fn add(str: String, data: Option<String>) -> BazaR<()> {
    Container::builder()
        .create_from_str(str)?
        .build()
        .create(data)?
        .commit()?;
    Ok(())
}

pub fn generate(str: String) -> BazaR<()> {
    let data = crate::generate(12, false, false, false)?;
    Container::builder()
        .create_from_str(str)?
        .build()
        .create(Some(data))?
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
    io::stdin()
        .read_to_string(&mut input)
        .or_raise(|| crate::error::Error::Message("Failed to read from stdin".into()))?;
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

    fn create(str: &str) {
        let str = str.to_string();
        let password = crate::generate(255, false, false, false).unwrap();
        Container::builder()
            .create_from_str(str)
            .unwrap()
            .build()
            .create(Some(password))
            .unwrap()
            .commit()
            .unwrap();
    }

    fn read(str: &str) {
        let str = str.to_string();
        Container::builder()
            .create_from_str(str)
            .unwrap()
            .build()
            .read()
            .unwrap();
    }

    fn delete(str: &str) {
        let str = str.to_string();
        Container::builder()
            .create_from_str(str)
            .unwrap()
            .build()
            .delete()
            .unwrap();
    }

    #[test]
    fn it_works() {
        let temp = tempfile::tempdir().unwrap();
        let config_path = temp.path().join("baza.toml");
        let mut config = Config::default();
        config.main.datadir = temp.path().to_string_lossy().to_string();
        let config_str = toml::to_string(&config).unwrap();
        std::fs::write(&config_path, config_str).unwrap();
        Config::build(&config_path).unwrap();

        let password = crate::generate(255, false, false, false).unwrap();
        init(Some(password.clone())).unwrap();
        cleanup_tmp_folder().unwrap();
        lock().unwrap();

        unlock(Some(password.clone())).unwrap();
        let bundles = vec![
            "test::my.test::login.ru",
            "test::my@test::login@ru",
            "test::my/test::login/ru",
            "test::my-test::login-ru",
            "test::my_test::login_ru",
        ];
        for name in bundles {
            create(name);
            read(name);
            delete(name);
        }
    }
}
