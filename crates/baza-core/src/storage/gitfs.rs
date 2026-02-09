use std::{
    fs::File,
    io::{BufRead, Read, Write},
    path::{PathBuf, MAIN_SEPARATOR},
    process::{exit, Command},
};

use arboard::Clipboard;
use colored::Colorize;
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
            let tree = add_to_index(&repo).map_err(|e| exn::Exn::new(e.into()))?;
            let signature = signature().map_err(|e| exn::Exn::new(e.into()))?;
            let parrent_commit = Some(head.peel_to_commit().map_err(|e| exn::Exn::new(e.into()))?);
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
            .map_err(|e| exn::Exn::new(e.into()))?;
        };
    } else {
        initialize()?;
        commit(msg)?;
    };

    Ok(())
}

pub fn initialize() -> BazaR<()> {
    let repo = Repository::init(super::storage_dir(DIR)).map_err(|e| exn::Exn::new(e.into()))?;
    let mut path = repo.path().to_path_buf();
    path.pop();
    let gitignore_file = format!("{}/.gitignore", &path.to_string_lossy());
    let mut file = File::create(gitignore_file).map_err(|e| exn::Exn::new(e.into()))?;
    let gitignore = r#""#;
    file.write_all(gitignore.trim().as_bytes())
        .map_err(|e| exn::Exn::new(e.into()))?;
    let tree = add_to_index(&repo).map_err(|e| exn::Exn::new(e.into()))?;
    let commit_message = "Initial commit";
    let signature = signature().map_err(|e| exn::Exn::new(e.into()))?;
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        commit_message,
        &tree,
        &[],
    )
    .map_err(|e| exn::Exn::new(e.into()))?;
    Ok(())
}

pub fn sync() -> BazaR<()> {
    let repo = Repository::open(super::storage_dir(DIR)).map_err(|e| exn::Exn::new(e.into()))?;

    let privatekey = if let Some(key) = &Config::get().gitfs.privatekey {
        key.clone()
    } else {
        format!(
            "{}/.ssh/id_ed25519",
            std::env::var("HOME").map_err(|e| exn::Exn::new(crate::error::Error::from(e)))?
        )
    };
    let passphrase = &Config::get().gitfs.passphrase;
    if let Some(url) = &Config::get().gitfs.url {
        let remote_name = "origin";
        if repo.find_remote(remote_name).is_err() {
            repo.remote(remote_name, url)
                .map_err(|e| exn::Exn::new(e.into()))?;
        }

        let mut remote = repo
            .find_remote(remote_name)
            .map_err(|e| exn::Exn::new(e.into()))?;

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
            .map_err(|e| exn::Exn::new(e.into()))?;

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
            std::fs::create_dir_all(parent).map_err(|e| exn::Exn::new(e.into()))?;
        }
        bundle
            .file
            .persist_noclobber(path)
            .map_err(|e| exn::Exn::new(e.into()))?;
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
            .map_err(|e| exn::Exn::new(e.into()))?;

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
            std::fs::remove_file(&path).map_err(|e| exn::Exn::new(e.into()))?;
        } else if path.is_dir() {
            std::fs::remove_dir_all(&path).map_err(|e| exn::Exn::new(e.into()))?;
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
            let entry = entry.map_err(|e| exn::Exn::new(e.into()))?;
            let path = entry.path();

            if path.is_file() {
                let path = path
                    .strip_prefix(&dir)
                    .map_err(|e| exn::Exn::new(e.into()))?
                    .with_extension("");
                let lossy = path
                    .to_string_lossy()
                    .replace(MAIN_SEPARATOR, &Config::get().main.box_delimiter);

                let re = regex::Regex::new(&pattern).map_err(|e| exn::Exn::new(e.into()))?;
                if re.is_match(&lossy) {
                    m(&format!("{lossy}\n"), MessageType::Clean);
                }
            }
        }
        Ok(())
    }

    fn copy_to_clipboard(&self, bundle: Bundle, ttl: u64) -> BazaR<()> {
        let mut clipboard = Clipboard::new().map_err(|e| exn::Exn::new(e.into()))?;
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

        let file = std::fs::File::open(file).map_err(|e| exn::Exn::new(e.into()))?;
        let mut buffer = std::io::BufReader::new(file);
        let mut first_line = String::new();
        buffer
            .read_line(&mut first_line)
            .map_err(|e| exn::Exn::new(e.into()))?;

        let lossy = first_line.trim();
        clipboard
            .set_text(lossy.trim())
            .map_err(|e| exn::Exn::new(e.into()))?;

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
