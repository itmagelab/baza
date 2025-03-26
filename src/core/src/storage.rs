pub mod gitfs;
// pub mod gix;

use std::path::PathBuf;

use crate::{BazaR, Config};

pub struct Ctx {
    pub name: String,
}

trait Storage {
    fn create(blob: &[u8], path: PathBuf, ctx: Option<Ctx>) -> BazaR<()>;
    fn read(file: PathBuf, load_from: PathBuf) -> BazaR<()>;
    fn update(file: PathBuf, load_from: PathBuf, ctx: Option<Ctx>) -> BazaR<()>;
    fn delete(path: PathBuf, ctx: Option<Ctx>) -> BazaR<()>;

    fn search(str: String) -> BazaR<()>;
    fn copy_to_clipboard(file: PathBuf, load_from: PathBuf, ttl: u64) -> BazaR<()>;
    fn initialize() -> BazaR<()> {
        tracing::warn!("initialize is not implemented");
        Ok(())
    }
    fn sync() -> BazaR<()> {
        tracing::warn!("syncing is not implemented");
        Ok(())
    }
}

pub fn initialize() -> BazaR<()> {
    if Config::get().gitfs.enable.unwrap_or(false) {
        gitfs::GitFs::initialize()?;
    }
    Ok(())
}

pub fn create(blob: &[u8], path: PathBuf, ctx: Option<Ctx>) -> BazaR<()> {
    if Config::get().gitfs.enable.unwrap_or(false) {
        gitfs::GitFs::create(blob, path, ctx)?;
    }
    Ok(())
}

pub fn sync() -> BazaR<()> {
    if Config::get().gitfs.enable.unwrap_or(false) {
        gitfs::GitFs::sync()?;
    }
    Ok(())
}

pub fn edit(file: PathBuf, load_from: PathBuf, ctx: Option<Ctx>) -> BazaR<()> {
    if Config::get().gitfs.enable.unwrap_or(false) {
        gitfs::GitFs::update(file, load_from, ctx)?;
    }
    Ok(())
}

pub fn show(file: PathBuf, load_from: PathBuf) -> BazaR<()> {
    if Config::get().gitfs.enable.unwrap_or(false) {
        gitfs::GitFs::read(file, load_from)?;
    }
    Ok(())
}

pub fn search(str: String) -> BazaR<()> {
    if Config::get().gitfs.enable.unwrap_or(false) {
        gitfs::GitFs::search(str)?;
    }
    Ok(())
}

pub fn delete(path: PathBuf, ctx: Option<Ctx>) -> BazaR<()> {
    if Config::get().gitfs.enable.unwrap_or(false) {
        gitfs::GitFs::delete(path, ctx)?;
    }
    Ok(())
}

pub fn copy_to_clipboard(file: PathBuf, load_from: PathBuf, ttl: u64) -> BazaR<()> {
    if Config::get().gitfs.enable.unwrap_or(false) {
        gitfs::GitFs::copy_to_clipboard(file, load_from, ttl)?;
    }
    Ok(())
}
