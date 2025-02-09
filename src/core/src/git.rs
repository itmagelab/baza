use std::{fs::File, io::Write};

use git2::{IndexAddOption, Repository, Signature, Tree};

use crate::{error::Error, BazaR, BAZA_DIR, DEFAULT_AUTHOR, DEFAULT_EMAIL};

fn signature() -> Result<Signature<'static>, git2::Error> {
    Signature::now(DEFAULT_AUTHOR, DEFAULT_EMAIL)
}

// TODO: Add index to struct (Repository)
fn add_to_index(repo: &'_ Repository) -> Result<Tree<'_>, git2::Error> {
    let mut index = repo.index()?;
    index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
    index.write()?;
    let tree_oid = index.write_tree()?;
    repo.find_tree(tree_oid)
}

pub fn commit(msg: String) -> BazaR<()> {
    let data = format!("{}/data", BAZA_DIR);
    let repo = Repository::init(&data).map_err(Error::Git2Error)?;
    if let Ok(head) = repo.head() {
        let tree = add_to_index(&repo).map_err(Error::Git2Error)?;
        let signature = signature().map_err(Error::Git2Error)?;
        let parrent_commit = Some(head.peel_to_commit().map_err(Error::Git2Error)?);
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &msg,
            &tree,
            &[&parrent_commit.ok_or(Error::CommonBazaError)?],
        )
        .map_err(Error::Git2Error)?;
    } else {
        initialize(&repo).map_err(Error::Git2Error)?;
        commit(msg)?;
    };
    Ok(())
}

fn initialize(repo: &Repository) -> Result<(), git2::Error> {
    let mut path = repo.path().to_path_buf();
    path.pop();
    let gitignore_file = format!("{}/.gitignore", &path.to_string_lossy().to_string());
    let mut file = match File::create(gitignore_file) {
        Ok(f) => f,
        Err(e) => panic!("Error creating gitignore file: {}", e),
    };
    let gitignore = r#""#;
    if let Err(e) = file.write_all(gitignore.trim().as_bytes()) {
        panic!("Error creating gitignore file: {}", e);
    };
    let tree = add_to_index(repo)?;
    let commit_message = "Initial commit";
    let signature = signature()?;
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        commit_message,
        &tree,
        &[],
    )?;
    Ok(())
}
