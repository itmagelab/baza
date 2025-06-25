pub mod gitfs;
pub mod gix;
pub mod sled;

use crate::{bundle::Bundle, BazaR, Config, Type};

pub fn storage_dir(dir: &'static str) -> std::path::PathBuf {
    std::path::PathBuf::from(format!("{}/data/{}", &Config::get().main.datadir, dir))
}

trait Storage {
    fn create(&self, bundle: Bundle, replace: bool) -> BazaR<()>;
    fn read(&self, bundle: Bundle) -> BazaR<()>;
    fn update(&self, bundle: Bundle) -> BazaR<()>;
    fn delete(&self, bundle: Bundle) -> BazaR<()>;
    fn search(&self, str: String) -> BazaR<()>;
    fn copy_to_clipboard(&self, bundle: Bundle, ttl: u64) -> BazaR<()>;
}

impl r#Type {
    pub fn initialize(&self) -> BazaR<()> {
        match self {
            crate::r#Type::Gitfs => gitfs::initialize()?,
            crate::r#Type::Gix => gix::initialize()?,
            crate::r#Type::Sled => todo!(),
        };
        Ok(())
    }
}

pub fn initialize() -> BazaR<()> {
    Config::get().storage.r#type.initialize()
}

pub(crate) fn create(bundle: Bundle) -> BazaR<()> {
    match Config::get().storage.r#type {
        crate::r#Type::Gitfs => gitfs::GitFs.create(bundle, true)?,
        crate::r#Type::Gix => gix::Gix.create(bundle, true)?,
        crate::r#Type::Sled => todo!(),
    };
    Ok(())
}

pub(crate) fn read(bundle: Bundle) -> BazaR<()> {
    match Config::get().storage.r#type {
        crate::r#Type::Gitfs => gitfs::GitFs.read(bundle)?,
        crate::r#Type::Gix => gix::Gix.read(bundle)?,
        crate::r#Type::Sled => todo!(),
    };
    Ok(())
}

pub(crate) fn update(bundle: Bundle) -> BazaR<()> {
    match Config::get().storage.r#type {
        crate::r#Type::Gitfs => gitfs::GitFs.update(bundle)?,
        crate::r#Type::Gix => (),
        crate::r#Type::Sled => todo!(),
    };
    Ok(())
}

pub(crate) fn delete(bundle: Bundle) -> BazaR<()> {
    match Config::get().storage.r#type {
        crate::r#Type::Gitfs => gitfs::GitFs.delete(bundle)?,
        crate::r#Type::Gix => (),
        crate::r#Type::Sled => todo!(),
    };
    Ok(())
}

pub fn sync() -> BazaR<()> {
    match Config::get().storage.r#type {
        crate::r#Type::Gitfs => gitfs::sync()?,
        crate::r#Type::Gix => (),
        crate::r#Type::Sled => todo!(),
    };
    Ok(())
}

pub fn search(str: String) -> BazaR<()> {
    match Config::get().storage.r#type {
        crate::r#Type::Gitfs => gitfs::GitFs.search(str)?,
        crate::r#Type::Gix => (),
        crate::r#Type::Sled => todo!(),
    };
    Ok(())
}

pub(crate) fn copy_to_clipboard(bundle: Bundle, ttl: u64) -> BazaR<()> {
    match Config::get().storage.r#type {
        crate::r#Type::Gitfs => gitfs::GitFs.copy_to_clipboard(bundle, ttl)?,
        crate::r#Type::Gix => (),
        crate::r#Type::Sled => todo!(),
    };
    Ok(())
}
