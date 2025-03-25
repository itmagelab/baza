pub mod git;

use std::path::PathBuf;

use tempfile::NamedTempFile;

use crate::{BazaR, Config};

trait Storage {
    fn initialize() -> BazaR<()>;
    fn commit(msg: String) -> BazaR<()>;
    fn push() -> BazaR<()>;
    fn add(file: NamedTempFile, path: PathBuf, replace: bool) -> BazaR<()>;
}

pub fn initialize() -> BazaR<()> {
    if Config::get().git.enable.unwrap_or(false) {
        git::Git::initialize()?;
    }
    Ok(())
}

pub fn commit(msg: String) -> BazaR<()> {
    if Config::get().git.enable.unwrap_or(false) {
        git::Git::commit(msg)?;
    }
    Ok(())
}

pub fn push() -> BazaR<()> {
    if Config::get().git.enable.unwrap_or(false) {
        git::Git::push()?;
    }
    Ok(())
}

pub fn add(file: NamedTempFile, path: PathBuf, replace: bool) -> BazaR<()> {
    if Config::get().git.enable.unwrap_or(false) {
        git::Git::add(file, path, replace)?;
    }
    Ok(())
}
