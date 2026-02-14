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

    async fn read(&mut self) -> BazaR<()> {
        let name = self.name();
        let content = storage::get_content(name).await?;
        #[cfg(not(target_arch = "wasm32"))]
        crate::m(&content, crate::MessageType::Clean);
        #[cfg(target_arch = "wasm32")]
        tracing::info!("Content: {}", content);
        Ok(())
    }

    async fn update(self) -> BazaR<Self> {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let name = self.name();
            let content = match storage::get_content(name.clone()).await {
                Ok(c) => c,
                Err(_) => String::new(),
            };
            
            let temp = tempfile::NamedTempFile::new().map_err(|e| crate::error::Error::Message(e.to_string()))?;
            std::fs::write(temp.path(), content).map_err(|e| crate::error::Error::Message(e.to_string()))?;

            let editor = std::env::var("EDITOR").unwrap_or(String::from("vi"));
            let status = std::process::Command::new(editor)
                .arg(temp.path())
                .status()
                .or_raise(|| crate::error::Error::Message("Failed to launch editor".into()))?;
            
            if !status.success() {
                std::process::exit(1);
            }

            let new_content = std::fs::read_to_string(temp.path()).map_err(|e| crate::error::Error::Message(e.to_string()))?;
            storage::save_content(name, new_content).await?;
        }
        Ok(self)
    }

    async fn delete(&mut self) -> BazaR<()> {
        let name = self.name();
        storage::delete_by_name(name).await?;
        Ok(())
    }

    async fn commit(&mut self, data: Option<String>) -> BazaR<()> {
        let name = self.name();
        if let Some(content) = data {
            storage::save_content(name, content).await?;
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
        // The last part is the bundle name. In create_from_str we add exactly one bundle.
        let bundle_name = self.bundles().pop().unwrap_or_default();
        name.push(bundle_name); 
        name.join(&Config::get().main.box_delimiter)
    }

    async fn copy_to_clipboard(self, ttl: u64) -> BazaR<()> {
        let name = self.name();
        storage::copy_to_clipboard(name, ttl).await?;
        Ok(())
    }
}

pub async fn add(str: String, data: Option<String>) -> BazaR<()> {
    let mut container = Container::builder().create_from_str(str)?.build();
    if let Some(content) = data {
        container.commit(Some(content)).await?;
    } else {
        container.update().await?;
    }
    Ok(())
}

pub async fn generate(str: String) -> BazaR<()> {
    let data = crate::generate(12, false, false, false)?;
    add(str, Some(data)).await
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
    add(str, Some(input)).await
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
        let password = match crate::generate(255, false, false, false) {
            Ok(p) => p,
            Err(e) => panic!("generate failed: {}", e),
        };
        match pollster::block_on(add(str, Some(password))) {
            Ok(_) => {}
            Err(e) => panic!("add failed: {}", e),
        }
    }

    fn read_test(str: &str) {
        let str = str.to_string();
        match pollster::block_on(read(str)) {
            Ok(_) => {}
            Err(e) => panic!("read failed: {}", e),
        }
    }

    fn delete_test(str: &str) {
        let str = str.to_string();
        match pollster::block_on(delete(str)) {
            Ok(_) => {}
            Err(e) => panic!("delete failed: {}", e),
        }
    }

    #[test]
    fn it_works() {
        let temp = match tempfile::tempdir() {
            Ok(t) => t,
            Err(e) => panic!("tempdir failed: {}", e),
        };
        let config_path = temp.path().join("baza.toml");
        let mut config = Config::default();
        config.main.datadir = temp.path().to_string_lossy().to_string();
        let config_str = match toml::to_string(&config) {
            Ok(s) => s,
            Err(e) => panic!("toml serialize failed: {}", e),
        };
        if let Err(e) = std::fs::write(&config_path, config_str) {
            panic!("write config failed: {}", e);
        }
        if let Err(e) = Config::build(&config_path) {
            panic!("Config::build failed: {}", e);
        }

        let password = match crate::generate(255, false, false, false) {
            Ok(p) => p,
            Err(e) => panic!("generate failed: {}", e),
        };
        if let Err(e) = init(Some(password.clone())) {
            panic!("init failed: {}", e);
        }
        if let Err(e) = cleanup_tmp_folder() {
            panic!("cleanup failed: {}", e);
        }
        if let Err(e) = lock() {
            panic!("lock failed: {}", e);
        }

        if let Err(e) = unlock(Some(password.clone())) {
            panic!("unlock failed: {}", e);
        }
        let bundles = vec![
            "test::my.test::login.ru",
            "test::my@test::login@ru",
            "test::my/test::login/ru",
            "test::my-test::login-ru",
            "test::my_test::login_ru",
        ];
        for name in bundles {
            create(name);
            read_test(name);
            delete_test(name);
        }
    }
}
