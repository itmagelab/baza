use std::{hash::Hash, io::Read, path::PathBuf};

use gix::{
    config::{
        tree::{Author, Committer},
        CommitAutoRollback, SnapshotMut,
    }, index::write::{Extensions, Options}, objs::{tree, Tree}
};

use crate::{
    encrypt_data,
    error::{self, Error},
    key, BazaR, Config, DEFAULT_AUTHOR, DEFAULT_EMAIL,
};

use super::{Bundle, Storage};

pub struct Gix;

const DIR: &str = "gix";

impl Gix {}

fn dir() -> PathBuf {
    PathBuf::from(format!("{}/data/{}", &Config::get().main.datadir, DIR))
}

fn extend_config(mut config: SnapshotMut) -> BazaR<CommitAutoRollback> {
    config.set_raw_value(&Author::NAME, DEFAULT_AUTHOR)?;
    config.set_raw_value(&Author::EMAIL, DEFAULT_EMAIL)?;
    config.set_raw_value(&Committer::NAME, DEFAULT_AUTHOR)?;
    config.set_raw_value(&Committer::EMAIL, DEFAULT_EMAIL)?;
    config
        .commit_auto_rollback()
        .map_err(|err| error::Error::GixConfig(Box::new(err)))
}

fn commit(msg: String, tree: Tree) -> BazaR<()> {
    let mut repo = gix::discover(dir()).map_err(|err| error::Error::GixDiscover(Box::new(err)))?;
    let repo = extend_config(repo.config_snapshot_mut())?;
    let Ok(commit) = repo.head() else {
        initialize()?;
        commit(msg, tree)?;
        return Ok(());
    };
    let tree_id = repo.write_object(&tree)?;
    let index = repo.index_or_load_from_head()?;
    // index.write(Options { extensions: Extensions::None, skip_hash: true });
    dbg!(repo.index_path());
    repo.commit("HEAD", msg, tree_id, commit.id())?;
    Ok(())
}

pub(crate) fn initialize() -> BazaR<()> {
    let mut repo = gix::init_bare(dir()).map_err(|err| error::Error::GixInit(Box::new(err)))?;

    let tree = gix::objs::Tree::empty();
    let empty_tree_id = repo.write_object(&tree)?.detach();

    let repo = extend_config(repo.config_snapshot_mut())?;
    repo.commit(
        "HEAD",
        "Initial commit",
        empty_tree_id,
        gix::commit::NO_PARENT_IDS,
    )?;

    Ok(())
}

impl Storage for Gix {
    fn create(&self, bundle: Bundle, _replace: bool) -> BazaR<()> {
        let ptr = bundle.ptr.ok_or(Error::NoPointerFound)?;
        let path: PathBuf = ptr.iter().collect();
        let name = ptr.join(&Config::get().main.box_delimiter);

        let repo = gix::discover(dir()).map_err(|err| error::Error::GixDiscover(Box::new(err)))?;
        let mut tree = gix::objs::Tree::empty();

        let mut buffer = Vec::new();
        let mut filename = std::fs::File::open(&bundle.file)?;
        filename.read_to_end(&mut buffer)?;
        let blob_id = repo.write_blob(encrypt_data(&buffer, &key()?)?)?.into();
        let entry = tree::Entry {
            mode: tree::EntryKind::Blob.into(),
            oid: blob_id,
            filename: path.to_string_lossy().to_string().into(),
        };
        tree.entries.push(entry);

        let msg = format!("Bundle {name} was added");
        commit(msg, tree)?;
        Ok(())
    }

    fn read(&self, bundle: super::Bundle) -> BazaR<()> {
        let ptr = bundle.ptr.ok_or(Error::NoPointerFound)?;
        let path: PathBuf = ptr.iter().collect();
        let name = ptr.join(&Config::get().main.box_delimiter);

        let repo = gix::discover(dir()).map_err(|err| error::Error::GixDiscover(Box::new(err)))?;

        let index = repo
            .index()
            .map_err(|err| Error::GixWorktreeOpen(Box::new(err)))?;
        for entry in index.entries() {
            dbg!(&entry);
        }

        let tree = repo.head_tree()?;
        let entry = tree
            .find_entry(path.to_string_lossy().to_string())
            .ok_or(Error::BundleNotExist(name.clone()))?;
        println!("{:?}", entry.filename());
        todo!()
    }

    fn update(&self, bundle: super::Bundle) -> BazaR<()> {
        todo!()
    }

    fn delete(&self, bundle: super::Bundle) -> BazaR<()> {
        todo!()
    }

    fn search(&self, str: String) -> BazaR<()> {
        todo!()
    }

    fn copy_to_clipboard(&self, bundle: super::Bundle, ttl: u64) -> BazaR<()> {
        todo!()
    }
}
