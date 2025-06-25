use std::io::Read;

use crate::{error::Error, storage::Storage, BazaR, Config};

use super::Bundle;

const DIR: &str = "sled";

pub struct Sled;

impl Sled {}

pub fn tree() -> BazaR<sled::Db> {
    let dir = super::storage_dir(DIR).join("db.sled");
    Ok(sled::open(dir)?)
}

impl Storage for Sled {
    fn create(&self, bundle: Bundle, _replace: bool) -> BazaR<()> {
        let mut content = String::new();
        bundle.file.as_file().read_to_string(&mut content)?;
        let ptr = bundle.ptr.ok_or(Error::NoPointerFound)?;
        let name = ptr.join(&Config::get().main.box_delimiter);
        tree()?.insert(name, content.as_str())?;
        Ok(())
    }

    fn read(&self, bundle: Bundle) -> BazaR<()> {
        Ok(())
    }

    fn update(&self, bundle: Bundle) -> BazaR<()> {
        Ok(())
    }

    fn delete(&self, bundle: Bundle) -> BazaR<()> {
        Ok(())
    }

    fn search(&self, str: String) -> BazaR<()> {
        Ok(())
    }

    fn copy_to_clipboard(&self, bundle: Bundle, ttl: u64) -> BazaR<()> {
        Ok(())
    }
}
