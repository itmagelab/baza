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
    container::ContainerBuilder, decrypt_file, encrypt_file, error::Error, m, storage::Storage,
    BazaR, Config, MessageType, DEFAULT_AUTHOR, DEFAULT_EMAIL, TTL_SECONDS,
};

use super::Ctx;

const DIR: &str = "gitfs";

pub struct GitFs;

impl GitFs {
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
        let data = format!("{}/data/{}", &Config::get().main.datadir, DIR);
        if let Ok(repo) = Repository::discover(&data) {
            if let Ok(head) = repo.head() {
                let tree = Self::add_to_index(&repo)?;
                let signature = Self::signature()?;
                let parrent_commit = Some(head.peel_to_commit()?);
                repo.commit(
                    Some("HEAD"),
                    &signature,
                    &signature,
                    &msg,
                    &tree,
                    &[&parrent_commit.ok_or(Error::CommonBazaError)?],
                )?;
            };
        } else {
            Self::initialize()?;
            Self::commit(msg)?;
        };

        Ok(())
    }
}

impl Storage for GitFs {
    fn initialize() -> BazaR<()> {
        let data = format!("{}/data/{}", &Config::get().main.datadir, DIR);
        let repo = Repository::init(&data)?;
        let mut path = repo.path().to_path_buf();
        path.pop();
        let gitignore_file = format!("{}/.gitignore", &path.to_string_lossy());
        let mut file = File::create(gitignore_file)?;
        let gitignore = r#""#;
        file.write_all(gitignore.trim().as_bytes())?;
        let tree = Self::add_to_index(&repo)?;
        let commit_message = "Initial commit";
        let signature = Self::signature()?;
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

    fn sync() -> BazaR<()> {
        let data = format!("{}/data/{}", &Config::get().main.datadir, DIR);
        let repo = Repository::open(&data)?;

        let privatekey = if let Some(key) = &Config::get().gitfs.privatekey {
            key.clone()
        } else {
            format!("{}/.ssh/id_ed25519", std::env::var("HOME")?)
        };
        let passphrase = &Config::get().gitfs.passphrase;
        if let Some(url) = &Config::get().gitfs.url {
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

    fn search(str: String) -> BazaR<()> {
        let datadir = PathBuf::from(format!("{}/data/{}", &Config::get().main.datadir, DIR));
        let builder = ContainerBuilder::new();
        let walker = WalkDir::new(&datadir).into_iter();
        for entry in walker.filter_entry(|e| !Self::is_hidden(e)) {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let path = path.strip_prefix(&datadir)?.with_extension("");
                let lossy = path
                    .to_string_lossy()
                    .replace(MAIN_SEPARATOR, &Config::get().main.box_delimiter);

                let re = regex::Regex::new(&str)?;
                if re.is_match(&lossy) {
                    let container = builder.clone().create_from_str(lossy)?.build();
                    m(&format!("{}\n", container), MessageType::Clean);
                }
            }
        }
        Ok(())
    }

    fn read(file: PathBuf, load_from: PathBuf) -> BazaR<()> {
        let datadir = PathBuf::from(format!("{}/data/{}", &Config::get().main.datadir, DIR));
        let load_from = datadir.join(load_from);

        std::fs::copy(load_from, &file)?;

        decrypt_file(&file)?;

        let mut file = File::open(file)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        m(&contents, crate::MessageType::Clean);
        Ok(())
    }

    fn update(file: PathBuf, load_from: PathBuf, ctx: Option<Ctx>) -> BazaR<()> {
        let datadir = PathBuf::from(format!("{}/data/{}", &Config::get().main.datadir, DIR));
        let load_from = datadir.join(load_from);

        let editor = std::env::var("EDITOR").unwrap_or(String::from("vi"));

        std::fs::copy(load_from, &file)?;

        decrypt_file(&file)?;

        let status = Command::new(editor).arg(&file).status()?;
        if !status.success() {
            exit(1);
        }

        encrypt_file(&file)?;

        let name = ctx.as_ref().map(|c| &c.name).ok_or(Error::NoName)?;

        let msg = format!("Bundle {name} was updated");
        Self::commit(msg)?;

        Ok(())
    }

    fn delete(path: PathBuf, ctx: Option<Ctx>) -> BazaR<()> {
        let datadir = PathBuf::from(format!("{}/data/{}", &Config::get().main.datadir, DIR));
        let delete_path = datadir.join(path);

        if delete_path.is_file() {
            std::fs::remove_file(&delete_path)?;
        } else if delete_path.is_dir() {
            std::fs::remove_dir_all(&delete_path)?;
        } else {
            return Ok(());
        };

        let name = ctx.as_ref().map(|c| &c.name).ok_or(Error::NoName)?;

        let msg = format!("Bundle {name} was deleted");
        Self::commit(msg)?;

        Ok(())
    }

    fn copy_to_clipboard(file: PathBuf, load_from: PathBuf, ttl: u64) -> BazaR<()> {
        let mut clipboard = Clipboard::new()?;
        let datadir = PathBuf::from(format!("{}/data/{}", &Config::get().main.datadir, DIR));
        let load_path = datadir.join(load_from);

        std::fs::copy(load_path, &file)?;

        decrypt_file(&file)?;

        let file = std::fs::File::open(file)?;
        let mut buffer = std::io::BufReader::new(file);
        let mut first_line = String::new();
        buffer.read_line(&mut first_line)?;

        let lossy = first_line.trim();
        clipboard.set_text(lossy.trim())?;

        let ttl_duration = std::time::Duration::new(ttl, 0);

        let message = format!(
            "Copied to clipboard. Will clear in {} seconds.",
            TTL_SECONDS
        );
        println!("{}", message.bright_yellow().bold());
        // TODO: This is start after sleep
        // m(&message, crate::MessageType::Data);
        std::thread::sleep(ttl_duration);
        clipboard.set_text("".to_string())?;
        Ok(())
    }

    fn create(blob: &[u8], path: PathBuf, ctx: Option<Ctx>) -> BazaR<()> {
        let datadir = PathBuf::from(format!("{}/data/{}", &Config::get().main.datadir, DIR));
        let new_path = datadir.join(path);
        if let Some(parent) = new_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = File::create(new_path)?;
        file.write_all(blob)?;

        let name = ctx.as_ref().map(|c| &c.name).ok_or(Error::NoName)?;

        let msg = format!("Bundle {name} was added");
        Self::commit(msg)?;

        Ok(())
    }
}
