use crate::{error::Error, BazaR, Config};

use super::{Bundle, Storage};

const DIR: &str = "gix";
const AUTHOR_NAME: &str = "Root Baza";
const AUTHOR_EMAIL: &str = "root@baza";

pub struct Gix;

impl Gix {}

fn set_config(config: &mut gix::config::SnapshotMut<'_>) -> BazaR<()> {
    config.set_raw_value(&gix::config::tree::Author::NAME, AUTHOR_NAME)?;
    config.set_raw_value(&gix::config::tree::Author::EMAIL, AUTHOR_EMAIL)?;
    config.set_raw_value(&gix::config::tree::Committer::NAME, AUTHOR_NAME)?;
    config.set_raw_value(&gix::config::tree::Committer::EMAIL, AUTHOR_EMAIL)?;
    Ok(())
}

pub(crate) fn initialize() -> BazaR<()> {
    let mut repo = gix::init_bare(super::storage_dir(DIR))?;
    let tree = gix::objs::Tree::empty();
    let empty_tree_id = repo.write_object(&tree)?.detach();
    let mut config = repo.config_snapshot_mut();
    set_config(&mut config)?;
    let repo = config.commit_auto_rollback()?;
    let msg = "initial commit";
    let initial_commit_id = repo.commit("HEAD", msg, empty_tree_id, gix::commit::NO_PARENT_IDS)?;
    tracing::debug!(?initial_commit_id);
    Ok(())
}

impl Storage for Gix {
    fn create(&self, bundle: Bundle, _replace: bool) -> BazaR<()> {
        let dir = super::storage_dir(DIR);
        let ptr = bundle.ptr.ok_or(Error::NoPointerFound)?;
        let name = ptr.join(&Config::get().main.box_delimiter);
        let mut repo = gix::open(dir)?;
        let file_data = std::fs::read(&bundle.file)?;
        let blob_id = repo.write_blob(&file_data)?.into();
        let mut tree = gix::objs::Tree::empty();

        // let tree = repo.head_tree()?;

        let entry = gix::objs::tree::Entry {
            mode: gix::objs::tree::EntryKind::Blob.into(),
            oid: blob_id,
            filename: name.clone().into(),
        };
        tree.entries.push(entry);
        let tree_id = repo.write_object(&tree)?.detach();
        let mut config = repo.config_snapshot_mut();
        set_config(&mut config)?;
        let repo = config.commit_auto_rollback()?;
        let msg = format!("Bundle {name} was added");
        let commit_id = repo.commit("HEAD", msg, tree_id, repo.head_id())?;
        tracing::debug!(?commit_id);
        Ok(())
    }

    fn read(&self, bundle: Bundle) -> BazaR<()> {
        tracing::debug!("list ehre");
        let dir = super::storage_dir(DIR);
        let ptr = bundle.ptr.ok_or(Error::NoPointerFound)?;
        let name = ptr.join(&Config::get().main.box_delimiter);
        let repo = gix::open(dir)?;
        let head_id = repo.head_id()?;
        let commit = repo.find_object(head_id)?.try_into_commit()?;
        let mut tree = commit.tree()?;
        let path = std::path::Path::new(&name);
        if let Some(entry) = tree.peel_to_entry_by_path(path)? {
            println!("✅ Найден файл: {:?}", path);
            println!("🔹 Blob ID: {}", entry.oid());
            println!("📦 File mode: {:?}", entry.mode());
        } else {
            println!("❌ Файл не найден: {:?}", path);
        }
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
