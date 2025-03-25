use gix::{config::tree::Author, objs::tree};

use crate::{error, BazaR, Config};

use super::Storage;

pub struct Gix;

impl Gix {}

impl Storage for Gix {
    fn initialize() -> BazaR<()> {
        let git_dir = format!("{}/data/gix", &Config::get().main.datadir);
        let mut repo =
            gix::init_bare(git_dir).map_err(|err| error::Error::GixInit(Box::new(err)))?;

        println!("Repo (bare): {:?}", repo.git_dir());

        let mut tree = gix::objs::Tree::empty();
        let empty_tree_id = repo.write_object(&tree)?.detach();

        let mut config = repo.config_snapshot_mut();
        config.set_raw_value(&Author::NAME, "Maria Sanchez")?;
        config.set_raw_value(&Author::EMAIL, "maria@example.com")?;
        {
            let repo = config
                .commit_auto_rollback()
                .map_err(|err| error::Error::GixConfig(Box::new(err)))?;
            let initial_commit_id = repo.commit(
                "HEAD",
                "initial commit",
                empty_tree_id,
                gix::commit::NO_PARENT_IDS,
            )?;

            println!("initial commit id with empty tree: {initial_commit_id:?}");

            let blob_id = repo.write_blob("hello world")?.into();
            let entry = tree::Entry {
                mode: tree::EntryKind::Blob.into(),
                oid: blob_id,
                filename: "hello.txt".into(),
            };

            tree.entries.push(entry);
            let hello_tree_id = repo.write_object(&tree)?;

            let blob_commit_id =
                repo.commit("HEAD", "hello commit", hello_tree_id, [initial_commit_id])?;

            println!("commit id for 'hello world' blob: {blob_commit_id:?}");
        }

        Ok(())
    }
    fn commit(_msg: String) -> BazaR<()> {
        Ok(())
    }
    fn push() -> BazaR<()> {
        Ok(())
    }
}
