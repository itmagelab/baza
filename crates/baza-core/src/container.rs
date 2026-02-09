use std::{cell::RefCell, fmt, rc::Rc};

use bundle::Bundle;
use exn::ResultExt;
use io::Read;
use r#box::BoxRef;

use super::*;
use crate::unlock;

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
            return Err(
                crate::error::Error::Message("Failed to parse container name".into()).into(),
            );
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

    async fn create(self, data: Option<String>) -> BazaR<Self> {
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

    async fn read(&mut self) -> BazaR<()> {
        if let Some(r#box) = self.boxes.last() {
            let box_name = r#box.borrow().name.to_string();
            let mut r#box = r#box.borrow_mut();
            let bundle = r#box.bundles.pop().ok_or_else(|| {
                crate::error::Error::Message(format!("The box {box_name} have not bundles"))
            })?;
            let bundle = Rc::try_unwrap(bundle)
                .map_err(|_| crate::error::Error::Message("Bundle still has references".into()))?
                .into_inner();
            storage::read(bundle).await?;
        }
        Ok(())
    }

    async fn update(self) -> BazaR<Self> {
        if let Some(r#box) = self.boxes.last() {
            let mut r#box = r#box.borrow_mut();
            let box_name = r#box.name.to_string();
            let bundle = r#box.bundles.pop().ok_or_else(|| {
                crate::error::Error::Message(format!("The box {box_name} have not bundles"))
            })?;
            let bundle = Rc::try_unwrap(bundle)
                .map_err(|_| crate::error::Error::Message("Bundle still has references".into()))?
                .into_inner();
            storage::update(bundle).await?;
        }
        Ok(self)
    }

    async fn delete(&mut self) -> BazaR<()> {
        while let Some(r#box) = self.boxes.pop() {
            let mut r#box = r#box.borrow_mut();
            while let Some(bundle) = r#box.bundles.pop() {
                let bundle = Rc::try_unwrap(bundle)
                    .map_err(|_| {
                        crate::error::Error::Message("Bundle still has references".into())
                    })?
                    .into_inner();
                storage::delete(bundle).await?;
            }
        }
        Ok(())
    }

    async fn commit(&mut self) -> BazaR<()> {
        while let Some(r#box) = self.boxes.pop() {
            let mut r#box = r#box.borrow_mut();
            while let Some(bundle) = r#box.bundles.pop() {
                let bundle = Rc::try_unwrap(bundle)
                    .map_err(|_| {
                        crate::error::Error::Message("Bundle still has references".into())
                    })?
                    .into_inner();
                storage::create(bundle).await?;
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

    async fn copy_to_clipboard(self, ttl: u64) -> BazaR<()> {
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
            storage::copy_to_clipboard(bundle, ttl).await?;
        }
        Ok(())
    }
}

pub async fn add(str: String, data: Option<String>) -> BazaR<()> {
    Container::builder()
        .create_from_str(str)?
        .build()
        .create(data)
        .await?
        .commit()
        .await?;
    Ok(())
}

pub async fn generate(str: String) -> BazaR<()> {
    let data = crate::generate(12, false, false, false)?;
    Container::builder()
        .create_from_str(str)?
        .build()
        .create(Some(data))
        .await?
        .commit()
        .await?;
    Ok(())
}

pub async fn read(str: String) -> BazaR<()> {
    Container::builder()
        .create_from_str(str)?
        .build()
        .read()
        .await?;
    Ok(())
}

pub async fn update(str: String) -> BazaR<()> {
    Container::builder()
        .create_from_str(str)?
        .build()
        .update()
        .await?;
    Ok(())
}

pub async fn delete(str: String) -> BazaR<()> {
    Container::builder()
        .create_from_str(str)?
        .build()
        .delete()
        .await?;
    Ok(())
}

pub async fn copy_to_clipboard(str: String) -> BazaR<()> {
    Container::builder()
        .create_from_str(str)?
        .build()
        .copy_to_clipboard(TTL_SECONDS)
        .await?;
    Ok(())
}

pub async fn from_stdin(str: String) -> BazaR<()> {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .or_raise(|| crate::error::Error::Message("Failed to read from stdin".into()))?;
    Container::builder()
        .create_from_str(str)?
        .build()
        .create(Some(input))
        .await?
        .commit()
        .await?;
    Ok(())
}

pub async fn search(str: String) -> BazaR<()> {
    storage::search(str).await?;
    Ok(())
}

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use super::*;
    use crate::unlock;

    fn create(str: &str) {
        let str = str.to_string();
        let password = crate::generate(255, false, false, false).unwrap();
        pollster::block_on(
            Container::builder()
                .create_from_str(str.clone())
                .unwrap()
                .build()
                .create(Some(password)),
        )
        .unwrap();
        pollster::block_on(
            pollster::block_on(
                Container::builder()
                    .create_from_str(str)
                    .unwrap()
                    .build()
                    .update(),
            ) // This is actually wrong in old code, but let's just fix the unwrap for now
            .unwrap()
            .commit(),
        )
        .unwrap();
    }

    fn read(str: &str) {
        let str = str.to_string();
        pollster::block_on(
            Container::builder()
                .create_from_str(str)
                .unwrap()
                .build()
                .read(),
        )
        .unwrap();
    }

    fn delete(str: &str) {
        let str = str.to_string();
        pollster::block_on(
            Container::builder()
                .create_from_str(str)
                .unwrap()
                .build()
                .delete(),
        )
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
