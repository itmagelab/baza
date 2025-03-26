use std::{
    io::Read,
    process::{exit, Command},
};

use gix::{
    config::tree::{Author, Committer},
    objs::tree,
};

use crate::{encrypt_data, error, key, BazaR, Config, DEFAULT_AUTHOR, DEFAULT_EMAIL};

use super::{Ctx, Storage};

pub struct Gix;

const DIR: &str = "gix";

impl Gix {
    fn _commit(msg: String) -> BazaR<()> {
        let git_dir = format!("{}/data/{}", &Config::get().main.datadir, DIR);
        let mut repo =
            gix::discover(git_dir).map_err(|err| error::Error::GixDiscover(Box::new(err)))?;
        let mut config = repo.config_snapshot_mut();
        config.set_raw_value(&Author::NAME, DEFAULT_AUTHOR)?;
        config.set_raw_value(&Author::EMAIL, DEFAULT_EMAIL)?;
        config.set_raw_value(&Committer::NAME, DEFAULT_AUTHOR)?;
        config.set_raw_value(&Committer::EMAIL, DEFAULT_EMAIL)?;
        let repo = config
            .commit_auto_rollback()
            .map_err(|err| error::Error::GixConfig(Box::new(err)))?;
        let Ok(tree) = repo.head_tree() else {
            Self::initialize()?;
            Self::_commit(msg)?;
            return Ok(());
        };
        repo.commit("HEAD", msg, tree.id(), repo.head()?.id())?;
        Ok(())
    }
}

impl Storage for Gix {
    fn initialize() -> BazaR<()> {
        let git_dir = format!("{}/data/{}", &Config::get().main.datadir, DIR);
        let mut repo =
            gix::init_bare(git_dir).map_err(|err| error::Error::GixInit(Box::new(err)))?;

        let tree = gix::objs::Tree::empty();
        let empty_tree_id = repo.write_object(&tree)?.detach();

        let mut config = repo.config_snapshot_mut();
        config.set_raw_value(&Author::NAME, DEFAULT_AUTHOR)?;
        config.set_raw_value(&Author::EMAIL, DEFAULT_EMAIL)?;
        config.set_raw_value(&Committer::NAME, DEFAULT_AUTHOR)?;
        config.set_raw_value(&Committer::EMAIL, DEFAULT_EMAIL)?;
        {
            let repo = config
                .commit_auto_rollback()
                .map_err(|err| error::Error::GixConfig(Box::new(err)))?;
            repo.commit(
                "HEAD",
                "Initial commit",
                empty_tree_id,
                gix::commit::NO_PARENT_IDS,
            )?;
        }

        Ok(())
    }

    fn create(file: std::path::PathBuf, str: Option<String>) -> BazaR<()> {
        let git_dir = format!("{}/data/{}", &Config::get().main.datadir, DIR);
        let repo =
            gix::discover(git_dir).map_err(|err| error::Error::GixDiscover(Box::new(err)))?;
        let mut tree = gix::objs::Tree::empty();

        let editor = std::env::var("EDITOR").unwrap_or(String::from("vi"));

        if let Some(str) = str {
            let blob_id = repo
                .write_blob(encrypt_data(str.as_bytes(), &key()?)?)?
                .into();
            let entry = tree::Entry {
                mode: tree::EntryKind::Blob.into(),
                oid: blob_id,
                filename: file.to_string_lossy().to_string().into(),
            };
            tree.entries.push(entry);
        } else {
            let status = Command::new(editor).arg(&file).status()?;
            if !status.success() {
                exit(1);
            }
            let mut buffer = Vec::new();
            let mut filename = std::fs::File::open(&file)?;
            filename.read_to_end(&mut buffer)?;
            let blob_id = repo.write_blob(encrypt_data(&buffer, &key()?)?)?.into();
            let entry = tree::Entry {
                mode: tree::EntryKind::Blob.into(),
                oid: blob_id,
                filename: file.to_string_lossy().to_string().into(),
            };
            tree.entries.push(entry);
        };

        repo.write_object(&tree)?;
        Ok(())
    }

    fn read(_file: std::path::PathBuf, _load_from: std::path::PathBuf) -> BazaR<()> {
        todo!()
    }

    fn update(
        _file: std::path::PathBuf,
        _load_from: std::path::PathBuf,
        _ctx: Option<Ctx>,
    ) -> BazaR<()> {
        todo!()
    }

    fn delete(_path: std::path::PathBuf, _ctx: Option<Ctx>) -> BazaR<()> {
        todo!()
    }

    fn search(_str: String) -> BazaR<()> {
        todo!()
    }

    fn copy_to_clipboard(
        _file: std::path::PathBuf,
        _load_from: std::path::PathBuf,
        _ttl: u64,
    ) -> BazaR<()> {
        todo!()
    }

    fn create(
        _file: tempfile::NamedTempFile,
        _path: std::path::PathBuf,
        _replace: bool,
        _ctx: Option<Ctx>,
    ) -> BazaR<()> {
        todo!()
    }
}
