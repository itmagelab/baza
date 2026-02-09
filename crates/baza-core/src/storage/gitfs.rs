use std::{
    fs::File,
    io::{BufRead, Read, Write},
    path::{PathBuf, MAIN_SEPARATOR},
    process::{exit, Command},
};

use arboard::Clipboard;
use colored::Colorize;
use exn::ResultExt;
use git2::{IndexAddOption, Repository, Signature, Tree};
use walkdir::{DirEntry, WalkDir};

use crate::{
    decrypt_file, encrypt_file, m, BazaR, Config, MessageType, DEFAULT_AUTHOR, DEFAULT_EMAIL,
    TTL_SECONDS,
};

use super::Bundle;

const DIR: &str = "gitfs";
const EXT: &str = "baza";

pub struct GitFs;

impl GitFs {}

fn signature() -> Result<Signature<'static>, git2::Error> {
    Signature::now(DEFAULT_AUTHOR, DEFAULT_EMAIL)
}

fn add_to_index(repo: &'_ Repository) -> Result<Tree<'_>, git2::Error> {
    let mut index = repo.index()?;
    index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;
    index.write()?;
    let tree_oid = index.write_tree()?;
    repo.find_tree(tree_oid)
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

fn commit(msg: String) -> BazaR<()> {
    if let Ok(repo) = Repository::discover(super::storage_dir(DIR)) {
        if let Ok(head) = repo.head() {
            let tree = add_to_index(&repo)
                .or_raise(|| crate::error::Error::Message("Failed to add to index".into()))?;
            let signature = signature()
                .or_raise(|| crate::error::Error::Message("Failed to get git signature".into()))?;
            let parrent_commit =
                Some(head.peel_to_commit().or_raise(|| {
                    crate::error::Error::Message("Failed to peel to commit".into())
                })?);
            repo.commit(
                Some("HEAD"),
                &signature,
                &signature,
                &msg,
                &tree,
                &[&parrent_commit.ok_or_else(|| {
                    crate::error::Error::Message("Parrent commit not found".into())
                })?],
            )
            .or_raise(|| crate::error::Error::Message("Failed to commit to repository".into()))?;
        };
    } else {
        initialize()?;
        commit(msg)?;
    };

    Ok(())
}

pub fn initialize() -> BazaR<()> {
    let repo = Repository::init(super::storage_dir(DIR))
        .or_raise(|| crate::error::Error::Message("Failed to initialize git repository".into()))?;
    let mut path = repo.path().to_path_buf();
    path.pop();
    let gitignore_file = format!("{}/.gitignore", &path.to_string_lossy());
    let mut file = File::create(gitignore_file)
        .or_raise(|| crate::error::Error::Message("Failed to create .gitignore file".into()))?;
    let gitignore = r#""#;
    file.write_all(gitignore.trim().as_bytes())
        .or_raise(|| crate::error::Error::Message("Failed to write to .gitignore".into()))?;
    let tree = add_to_index(&repo)
        .or_raise(|| crate::error::Error::Message("Failed to add to index".into()))?;
    let commit_message = "Initial commit";
    let signature = signature()
        .or_raise(|| crate::error::Error::Message("Failed to get git signature".into()))?;
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        commit_message,
        &tree,
        &[],
    )
    .or_raise(|| crate::error::Error::Message("Failed to commit initial changes".into()))?;
    Ok(())
}

pub fn sync() -> BazaR<()> {
    let repo = Repository::open(super::storage_dir(DIR))
        .or_raise(|| crate::error::Error::Message("Failed to open git repository".into()))?;

    let privatekey = if let Some(key) = &Config::get().gitfs.privatekey {
        key.clone()
    } else {
        format!(
            "{}/.ssh/id_ed25519",
            std::env::var("HOME").or_raise(|| crate::error::Error::Message(
                "Failed to get HOME environment variable".into()
            ))?
        )
    };
    let passphrase = &Config::get().gitfs.passphrase;
    if let Some(url) = &Config::get().gitfs.url {
        let remote_name = "origin";
        if repo.find_remote(remote_name).is_err() {
            repo.remote(remote_name, url)
                .or_raise(|| crate::error::Error::Message("Failed to add git remote".into()))?;
        }

        let mut remote = repo
            .find_remote(remote_name)
            .or_raise(|| crate::error::Error::Message("Failed to find git remote".into()))?;

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

        remote
            .push(
                &[&format!("refs/heads/{}", "master")],
                Some(&mut push_options),
            )
            .or_raise(|| {
                crate::error::Error::Message("Failed to push to remote repository".into())
            })?;

        tracing::info!("Pushed successfully");
    };

    Ok(())
}

impl crate::storage::StorageBackend for GitFs {
    fn sync(&self) -> BazaR<()> {
        sync()
    }

    fn create(&self, bundle: Bundle, _replace: bool) -> BazaR<()> {
        let ptr = bundle
            .ptr
            .ok_or_else(|| crate::error::Error::Message("Pointer not found".into()))?;
        let filename = ptr
            .last()
            .ok_or_else(|| crate::error::Error::Message("Must specify at least one".into()))?;
        let path: PathBuf = ptr.iter().collect();
        let name = ptr.join(&Config::get().main.box_delimiter);
        let path = super::storage_dir(DIR)
            .join(path)
            .with_file_name(format!("{filename}.{EXT}"));
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).or_raise(|| {
                crate::error::Error::Message("Failed to create parent directory".into())
            })?;
        }
        bundle
            .file
            .persist_noclobber(path)
            .or_raise(|| crate::error::Error::Message("Failed to persist bundle file".into()))?;
        let msg = format!("Bundle {name} was added");
        commit(msg)?;
        Ok(())
    }

    fn read(&self, bundle: Bundle) -> BazaR<()> {
        let ptr = bundle
            .ptr
            .ok_or_else(|| crate::error::Error::Message("Pointer not found".into()))?;
        let filename = ptr
            .last()
            .ok_or_else(|| crate::error::Error::Message("Must specify at least one".into()))?;
        let path: PathBuf = ptr.iter().collect();
        let path = super::storage_dir(DIR)
            .join(path)
            .with_file_name(format!("{filename}.{EXT}"));
        let file = bundle.file.path().to_path_buf();

        std::fs::copy(path, &file).map_err(|e| exn::Exn::new(e.into()))?;

        decrypt_file(&file)?;

        let mut file = File::open(file).map_err(|e| exn::Exn::new(e.into()))?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .map_err(|e| exn::Exn::new(e.into()))?;

        m(&contents, crate::MessageType::Clean);
        Ok(())
    }

    fn update(&self, bundle: Bundle) -> BazaR<()> {
        let ptr = bundle
            .ptr
            .ok_or_else(|| crate::error::Error::Message("Pointer not found".into()))?;
        let filename = ptr
            .last()
            .ok_or_else(|| crate::error::Error::Message("Must specify at least one".into()))?;
        let path: PathBuf = ptr.iter().collect();
        let name = ptr.join(&Config::get().main.box_delimiter);
        let path = super::storage_dir(DIR)
            .join(path)
            .with_file_name(format!("{filename}.{EXT}"));
        let file = bundle.file.path().to_path_buf();

        let editor = std::env::var("EDITOR").unwrap_or(String::from("vi"));

        std::fs::copy(path.clone(), &file).map_err(|e| exn::Exn::new(e.into()))?;

        decrypt_file(&file)?;

        let status = Command::new(editor)
            .arg(&file)
            .status()
            .map_err(|e| exn::Exn::new(e.into()))?;
        if !status.success() {
            exit(1);
        }

        encrypt_file(&file)?;

        bundle
            .file
            .persist(path)
            .or_raise(|| crate::error::Error::Message("Failed to persist updated bundle".into()))?;

        let msg = format!("Bundle {name} was updated");
        commit(msg)?;

        Ok(())
    }

    fn delete(&self, bundle: Bundle) -> BazaR<()> {
        let ptr = bundle
            .ptr
            .ok_or_else(|| crate::error::Error::Message("Pointer not found".into()))?;
        let filename = ptr
            .last()
            .ok_or_else(|| crate::error::Error::Message("Must specify at least one".into()))?;
        let path: PathBuf = ptr.iter().collect();
        let name = ptr.join(&Config::get().main.box_delimiter);
        let path = super::storage_dir(DIR)
            .join(path)
            .with_file_name(format!("{filename}.{EXT}"));

        if path.is_file() {
            std::fs::remove_file(&path)
                .or_raise(|| crate::error::Error::Message("Failed to remove bundle file".into()))?;
        } else if path.is_dir() {
            std::fs::remove_dir_all(&path).or_raise(|| {
                crate::error::Error::Message("Failed to remove bundle directory".into())
            })?;
        } else {
            return Ok(());
        };

        let msg = format!("Bundle {name} was deleted");
        commit(msg)?;

        Ok(())
    }

    fn search(&self, pattern: String) -> BazaR<()> {
        let dir = super::storage_dir(DIR);
        let walker = WalkDir::new(&dir).into_iter();
        for entry in walker.filter_entry(|e| !is_hidden(e)) {
            let entry = entry.or_raise(|| {
                crate::error::Error::Message("Failed to read directory entry".into())
            })?;
            let path = entry.path();

            if path.is_file() {
                let path = path
                    .strip_prefix(&dir)
                    .or_raise(|| {
                        crate::error::Error::Message("Failed to strip directory prefix".into())
                    })?
                    .with_extension("");
                let lossy = path
                    .to_string_lossy()
                    .replace(MAIN_SEPARATOR, &Config::get().main.box_delimiter);

                let re = regex::Regex::new(&pattern)
                    .or_raise(|| crate::error::Error::Message("Invalid search pattern".into()))?;
                if re.is_match(&lossy) {
                    m(&format!("{lossy}\n"), MessageType::Clean);
                }
            }
        }
        Ok(())
    }

    fn copy_to_clipboard(&self, bundle: Bundle, ttl: u64) -> BazaR<()> {
        let mut clipboard = Clipboard::new()
            .or_raise(|| crate::error::Error::Message("Failed to access clipboard".into()))?;
        let ptr = bundle
            .ptr
            .ok_or_else(|| crate::error::Error::Message("Pointer not found".into()))?;
        let filename = ptr
            .last()
            .ok_or_else(|| crate::error::Error::Message("Must specify at least one".into()))?;
        let path: PathBuf = ptr.iter().collect();
        let path = super::storage_dir(DIR)
            .join(path)
            .with_file_name(format!("{filename}.{EXT}"));
        let file = bundle.file.path().to_path_buf();

        std::fs::copy(path, &file).or_raise(|| {
            crate::error::Error::Message("Failed to copy bundle to temporary file".into())
        })?;

        decrypt_file(&file)?;

        let file = std::fs::File::open(file)
            .or_raise(|| crate::error::Error::Message("Failed to open temporary file".into()))?;
        let mut buffer = std::io::BufReader::new(file);
        let mut first_line = String::new();
        buffer
            .read_line(&mut first_line)
            .or_raise(|| crate::error::Error::Message("Failed to read secrets file".into()))?;

        let lossy = first_line.trim();
        clipboard
            .set_text(lossy.trim())
            .or_raise(|| crate::error::Error::Message("Failed to set clipboard text".into()))?;

        let ttl_duration = std::time::Duration::new(ttl, 0);

        let message = format!("Copied to clipboard. Will clear in {TTL_SECONDS} seconds.");
        println!("{}", message.bright_yellow().bold());
        // TODO: This is start after sleep
        // m(&message, crate::MessageType::Data);
        std::thread::sleep(ttl_duration);
        clipboard
            .set_text("".to_string())
            .map_err(|e| exn::Exn::new(e.into()))?;
        Ok(())
    }
}
