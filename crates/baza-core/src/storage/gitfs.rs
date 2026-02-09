use std::{
    fs::File,
    io::{BufRead, Read, Write},
    path::{PathBuf, MAIN_SEPARATOR},
    process::{exit, Command},
};

use arboard::Clipboard;
use colored::Colorize;
use exn::ResultExt;
use walkdir::{DirEntry, WalkDir};

use crate::{
    decrypt_file, encrypt_file, m, BazaR, Config, MessageType, DEFAULT_AUTHOR, DEFAULT_EMAIL,
    TTL_SECONDS,
};

use super::Bundle;

const DIR: &str = "gitfs";
const EXT: &str = "baza";

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}

pub struct GitFs;

impl GitFs {}

fn git_cmd() -> Command {
    let mut cmd = Command::new("git");
    cmd.current_dir(super::storage_dir(DIR));
    cmd
}

fn commit(msg: String) -> BazaR<()> {
    let storage_dir = super::storage_dir(DIR);
    if !storage_dir.exists() || !storage_dir.join(".git").exists() {
        initialize()?;
    }

    git_cmd()
        .args(["add", "."])
        .status()
        .or_raise(|| crate::error::Error::Message("Failed to add to index".into()))?;

    let status = git_cmd()
        .args(["commit", "-m", &msg])
        .status()
        .or_raise(|| crate::error::Error::Message("Failed to commit to repository".into()))?;

    if !status.success() {
        tracing::debug!("Nothing to commit or commit failed");
    }

    Ok(())
}

pub fn initialize() -> BazaR<()> {
    let storage_dir = super::storage_dir(DIR);
    std::fs::create_dir_all(&storage_dir)
        .or_raise(|| crate::error::Error::Message("Failed to create storage directory".into()))?;

    Command::new("git")
        .arg("init")
        .current_dir(&storage_dir)
        .status()
        .or_raise(|| crate::error::Error::Message("Failed to initialize git repository".into()))?;

    let gitignore_file = storage_dir.join(".gitignore");
    let mut file = File::create(gitignore_file)
        .or_raise(|| crate::error::Error::Message("Failed to create .gitignore file".into()))?;
    file.write_all(b"")
        .or_raise(|| crate::error::Error::Message("Failed to write to .gitignore".into()))?;

    Command::new("git")
        .args(["config", "user.name", DEFAULT_AUTHOR])
        .current_dir(&storage_dir)
        .status()
        .or_raise(|| crate::error::Error::Message("Failed to set git user.name".into()))?;

    Command::new("git")
        .args(["config", "user.email", DEFAULT_EMAIL])
        .current_dir(&storage_dir)
        .status()
        .or_raise(|| crate::error::Error::Message("Failed to set git user.email".into()))?;

    git_cmd()
        .args(["add", ".gitignore"])
        .status()
        .or_raise(|| crate::error::Error::Message("Failed to add .gitignore".into()))?;

    git_cmd()
        .args(["commit", "-m", "Initial commit"])
        .status()
        .or_raise(|| crate::error::Error::Message("Failed to commit initial changes".into()))?;

    Ok(())
}

pub fn sync() -> BazaR<()> {
    if let Some(url) = &Config::get().gitfs.url {
        // Add remote if not exists
        let remotes = git_cmd()
            .args(["remote"])
            .output()
            .or_raise(|| crate::error::Error::Message("Failed to list remotes".into()))?;
        let remotes_str = String::from_utf8_lossy(&remotes.stdout);

        if !remotes_str.contains("origin") {
            git_cmd()
                .args(["remote", "add", "origin", url])
                .status()
                .or_raise(|| crate::error::Error::Message("Failed to add git remote".into()))?;
        }

        // Push to remote
        let status = git_cmd()
            .args(["push", "origin", "master"])
            .status()
            .or_raise(|| {
                crate::error::Error::Message("Failed to push to remote repository".into())
            })?;

        if status.success() {
            tracing::info!("Pushed successfully");
        } else {
            exn::bail!(crate::error::Error::Message(
                "Failed to push to remote repository".into()
            ));
        }
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

                let re = regex_lite::Regex::new(&pattern)
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
