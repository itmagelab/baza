pub mod gitfs;

use std::path::PathBuf;

use tempfile::NamedTempFile;

use crate::{BazaR, Config};

trait Storage {
    fn create(file: PathBuf, str: Option<String>) -> BazaR<()>;
    fn read(file: PathBuf, load_from: PathBuf) -> BazaR<()>;
    fn update(file: PathBuf, load_from: PathBuf) -> BazaR<()>;
    fn delete(path: PathBuf) -> BazaR<()>;
    fn search(str: String) -> BazaR<()>;
    fn copy_to_clipboard(file: PathBuf, load_from: PathBuf, ttl: u64) -> BazaR<()>;
    fn initialize() -> BazaR<()> {
        tracing::warn!("initialize is not implemented");
        Ok(())
    }
    fn commit(_msg: String) -> BazaR<()> {
        tracing::warn!("commit is not implemented");
        Ok(())
    }
    fn sync() -> BazaR<()> {
        tracing::warn!("syncing is not implemented");
        Ok(())
    }
    fn save(_file: NamedTempFile, _path: PathBuf, _replace: bool) -> BazaR<()> {
        tracing::warn!("saving is not implemented");
        Ok(())
    }
}

pub fn initialize() -> BazaR<()> {
    if Config::get().gitfs.enable.unwrap_or(false) {
        gitfs::GitFs::initialize()?;
    }
    Ok(())
}

pub fn commit(msg: String) -> BazaR<()> {
    if Config::get().gitfs.enable.unwrap_or(false) {
        gitfs::GitFs::commit(msg)?;
    }
    Ok(())
}

pub fn sync() -> BazaR<()> {
    if Config::get().gitfs.enable.unwrap_or(false) {
        gitfs::GitFs::sync()?;
    }
    Ok(())
}

pub fn save(file: NamedTempFile, path: PathBuf, replace: bool) -> BazaR<()> {
    if Config::get().gitfs.enable.unwrap_or(false) {
        gitfs::GitFs::save(file, path, replace)?;
    }
    Ok(())
}

pub fn edit(file: PathBuf, load_from: PathBuf) -> BazaR<()> {
    if Config::get().gitfs.enable.unwrap_or(false) {
        gitfs::GitFs::update(file, load_from)?;
    }
    Ok(())
}

pub fn show(file: PathBuf, load_from: PathBuf) -> BazaR<()> {
    if Config::get().gitfs.enable.unwrap_or(false) {
        gitfs::GitFs::read(file, load_from)?;
    }
    Ok(())
}

pub fn create(file: PathBuf, str: Option<String>) -> BazaR<()> {
    if Config::get().gitfs.enable.unwrap_or(false) {
        gitfs::GitFs::create(file, str)?;
    }
    Ok(())
}

pub fn search(str: String) -> BazaR<()> {
    if Config::get().gitfs.enable.unwrap_or(false) {
        gitfs::GitFs::search(str)?;
    }
    Ok(())
}

pub fn delete(path: PathBuf) -> BazaR<()> {
    if Config::get().gitfs.enable.unwrap_or(false) {
        gitfs::GitFs::delete(path)?;
    }
    Ok(())
}

pub fn copy_to_clipboard(file: PathBuf, load_from: PathBuf, ttl: u64) -> BazaR<()> {
    if Config::get().gitfs.enable.unwrap_or(false) {
        gitfs::GitFs::copy_to_clipboard(file, load_from, ttl)?;
    }
    Ok(())
}
