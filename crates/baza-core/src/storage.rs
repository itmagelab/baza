pub mod gitfs;
pub mod redb;

use crate::{bundle::Bundle, BazaR, Config};

pub fn storage_dir(dir: &'static str) -> std::path::PathBuf {
    std::path::PathBuf::from(format!("{}/data/{}", &Config::get().main.datadir, dir))
}

pub(crate) trait StorageBackend {
    fn create(&self, bundle: Bundle, replace: bool) -> BazaR<()>;
    fn read(&self, bundle: Bundle) -> BazaR<()>;
    fn update(&self, bundle: Bundle) -> BazaR<()>;
    fn delete(&self, bundle: Bundle) -> BazaR<()>;
    fn search(&self, pattern: String) -> BazaR<()>;
    fn copy_to_clipboard(&self, bundle: Bundle, ttl: u64) -> BazaR<()>;
    fn sync(&self) -> BazaR<()>;
}

fn with_backend<F, R>(f: F) -> BazaR<R>
where
    F: FnOnce(&dyn StorageBackend) -> BazaR<R>,
{
    match Config::get().storage.r#type {
        crate::r#Type::Gitfs => f(&gitfs::GitFs),
        crate::r#Type::Redb => f(&redb::Redb::instance()?),
    }
}

pub fn initialize() -> BazaR<()> {
    match Config::get().storage.r#type {
        crate::r#Type::Gitfs => gitfs::initialize()?,
        crate::r#Type::Redb => redb::initialize()?,
    };
    Ok(())
}

pub(crate) fn create(bundle: Bundle) -> BazaR<()> {
    with_backend(|backend| backend.create(bundle, true))
}

pub(crate) fn read(bundle: Bundle) -> BazaR<()> {
    with_backend(|backend| backend.read(bundle))
}

pub(crate) fn update(bundle: Bundle) -> BazaR<()> {
    with_backend(|backend| backend.update(bundle))
}

pub(crate) fn delete(bundle: Bundle) -> BazaR<()> {
    with_backend(|backend| backend.delete(bundle))
}

pub fn sync() -> BazaR<()> {
    with_backend(|backend| backend.sync())
}

pub fn search(str: String) -> BazaR<()> {
    with_backend(|backend| backend.search(str))
}

pub(crate) fn copy_to_clipboard(bundle: Bundle, ttl: u64) -> BazaR<()> {
    with_backend(|backend| backend.copy_to_clipboard(bundle, ttl))
}
