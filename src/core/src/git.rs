use std::{fs::File, io::Write};

use git2::{IndexAddOption, Repository, Signature, Tree};

use crate::{error::Error, BazaR, Config, DEFAULT_AUTHOR, DEFAULT_EMAIL};

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

pub fn push() -> BazaR<()> {
    let data = format!("{}/data", &Config::get().main.datadir);
    let repo = Repository::open(&data)?;

    let privatekey = if let Some(key) = &Config::get().git.privatekey {
        key.clone()
    } else {
        format!("{}/.ssh/id_ed25519", std::env::var("HOME")?)
    };
    let passphrase = &Config::get().git.passphrase;
    if let Some(url) = &Config::get().git.url {
        let remote_name = "origin";
        if repo.find_remote(remote_name).is_err() {
            repo.remote(remote_name, url)?;
        }

        let mut remote = repo.find_remote(remote_name)?;

        let mut callbacks = git2::RemoteCallbacks::new();
        callbacks.credentials(|_, username_from_url, _| {
            git2::Cred::ssh_key(
                username_from_url.unwrap_or("git"),
                None,
                std::path::Path::new(privatekey.as_str()),
                passphrase.as_deref(),
            )
        });

        let mut push_options = git2::PushOptions::new();
        push_options.remote_callbacks(callbacks);

        remote.push(
            &[&format!("refs/heads/{}", "master")],
            Some(&mut push_options),
        )?;

        tracing::info!("Pushed successfully");
    };

    Ok(())
}

pub fn commit(msg: String) -> BazaR<()> {
    let data = format!("{}/data", &Config::get().main.datadir);
    let repo = Repository::init(&data)?;

    if let Ok(head) = repo.head() {
        let tree = add_to_index(&repo)?;
        let signature = signature()?;
        let parrent_commit = Some(head.peel_to_commit()?);
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &msg,
            &tree,
            &[&parrent_commit.ok_or(Error::CommonBazaError)?],
        )?;
    } else {
        initialize(&repo)?;
        commit(msg)?;
    };

    Ok(())
}

fn initialize(repo: &Repository) -> BazaR<()> {
    let mut path = repo.path().to_path_buf();
    path.pop();
    let gitignore_file = format!("{}/.gitignore", &path.to_string_lossy());
    let mut file = File::create(gitignore_file)?;
    let gitignore = r#""#;
    file.write_all(gitignore.trim().as_bytes())?;
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
