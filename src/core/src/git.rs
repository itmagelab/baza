use std::{fs::File, io::Write};

use colored::Colorize;
use git2::{IndexAddOption, Repository, Signature};

use crate::{error::Error, BazaR, BAZA_DIR, DEFAULT_AUTHOR, DEFAULT_EMAIL};

pub fn init() -> BazaR<()> {
    let data = format!("{}/data", BAZA_DIR);
    let repo = Repository::init(&data).map_err(Error::Git2Error)?;
    let gitignore_file = format!("{}/.gitignore", &data);
    let mut file = File::create(gitignore_file)?;
    let gitignore = r#""#;
    file.write_all(gitignore.trim().as_bytes())?;
    let mut index = repo.index().map_err(Error::Git2Error)?;
    index
        .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
        .map_err(Error::Git2Error)?;
    index.write().map_err(Error::Git2Error)?;
    let tree_oid = index.write_tree().map_err(Error::Git2Error)?;
    let tree = repo.find_tree(tree_oid).map_err(Error::Git2Error)?;
    let parent_commit = match repo.head() {
        Ok(head) => Some(head.peel_to_commit().map_err(Error::Git2Error)?),
        Err(_) => None,
    };
    let commit_message = "Initial commit";
    let signature = Signature::now(DEFAULT_AUTHOR, DEFAULT_EMAIL).map_err(Error::Git2Error)?;
    match parent_commit {
        Some(_) => {
            let message = "Repository already has commits";
            println!("{}", message.bright_yellow().bold());
        }
        None => {
            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                commit_message,
                &tree,
                &[],
            )
            .map_err(Error::Git2Error)?;
        }
    };
    Ok(())
}

pub fn commit(msg: String) -> BazaR<()> {
    let data = format!("{}/data", BAZA_DIR);
    let repo = Repository::init(data).map_err(Error::Git2Error)?;
    let mut index = repo.index().map_err(Error::Git2Error)?;
    index
        .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
        .map_err(Error::Git2Error)?;
    index.write().map_err(Error::Git2Error)?;
    let tree_oid = index.write_tree().map_err(Error::Git2Error)?;
    let tree = repo.find_tree(tree_oid).map_err(Error::Git2Error)?;
    let commit_message = format!("Bundle {} was changed", msg);

    let parent_commit = match repo.head() {
        Ok(head) => Some(head.peel_to_commit().map_err(Error::Git2Error)?),
        Err(_) => None,
    };
    let signature = Signature::now(DEFAULT_AUTHOR, DEFAULT_EMAIL).map_err(Error::Git2Error)?;
    match parent_commit {
        Some(parent) => {
            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                &commit_message,
                &tree,
                &[&parent],
            )
            .map_err(Error::Git2Error)?;
        }
        None => {
            tracing::debug!("Need initial commit");
        }
    };
    Ok(())
}
