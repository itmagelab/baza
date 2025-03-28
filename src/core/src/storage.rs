pub mod gitfs;

use crate::{bundle::Bundle, BazaR, Config};

trait Storage {
    fn create(&self, bundle: Bundle, replace: bool) -> BazaR<()>;
    fn read(&self, bundle: Bundle) -> BazaR<()>;
    fn update(&self, bundle: Bundle) -> BazaR<()>;
    fn delete(&self, bundle: Bundle) -> BazaR<()>;
    fn search(&self, str: String) -> BazaR<()>;
    fn copy_to_clipboard(&self, bundle: Bundle, ttl: u64) -> BazaR<()>;
}

pub fn initialize() -> BazaR<()> {
    if Config::get().gitfs.enable.unwrap_or(false) {
        gitfs::initialize()?;
    }
    Ok(())
}

pub(crate) fn create(bundle: Bundle) -> BazaR<()> {
    if Config::get().gitfs.enable.unwrap_or(false) {
        gitfs::GitFs.create(bundle, true)?;
    }
    Ok(())
}

pub(crate) fn read(bundle: Bundle) -> BazaR<()> {
    if Config::get().gitfs.enable.unwrap_or(false) {
        gitfs::GitFs.read(bundle)?;
    }
    Ok(())
}

pub(crate) fn update(bundle: Bundle) -> BazaR<()> {
    if Config::get().gitfs.enable.unwrap_or(false) {
        gitfs::GitFs.update(bundle)?;
    }
    Ok(())
}

pub(crate) fn delete(bundle: Bundle) -> BazaR<()> {
    if Config::get().gitfs.enable.unwrap_or(false) {
        gitfs::GitFs.delete(bundle)?;
    }
    Ok(())
}

pub fn sync() -> BazaR<()> {
    if Config::get().gitfs.enable.unwrap_or(false) {
        gitfs::sync()?;
    }
    Ok(())
}

pub fn search(str: String) -> BazaR<()> {
    if Config::get().gitfs.enable.unwrap_or(false) {
        gitfs::GitFs.search(str)?;
    }
    Ok(())
}

pub(crate) fn copy_to_clipboard(bundle: Bundle, ttl: u64) -> BazaR<()> {
    if Config::get().gitfs.enable.unwrap_or(false) {
        gitfs::GitFs.copy_to_clipboard(bundle, ttl)?;
    }
    Ok(())
}
