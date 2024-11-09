use crate::error::Error;
use std::io::Write;
use std::{
    env,
    fs::{self, File},
    io::Read,
    path::PathBuf,
    process::{exit, Command},
};
use tempfile::NamedTempFile;

#[derive(Debug, Clone, Default)]
pub struct Bundle {
    pub name: String,
    pub path: PathBuf,
}

impl Bundle {
    pub(crate) fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    pub fn write(self, content: &str) -> Result<Self, Error> {
        let mut file = File::create(&self.path)?;
        file.write_all(content.as_bytes())?;
        Ok(self)
    }

    pub fn read(&self) -> Result<String, Error> {
        let mut buffer = String::new();
        let mut file = File::open(&self.path)?;
        file.read_to_string(&mut buffer)?;
        Ok(buffer)
    }

    pub fn copy(&self, path: &PathBuf) -> Result<(), Error> {
        fs::copy(&self.path, path)?;
        Ok(())
    }

    pub fn edit(self) -> Result<Self, Error> {
        let editor = env::var("EDITOR").unwrap_or(String::from("vi"));
        let temp_file = NamedTempFile::new()?;
        if self.path.as_path().exists() {
            self.copy(&temp_file.path().to_path_buf())?;
        } else if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        };

        let temp_file_path = temp_file.path().as_os_str();
        let status = Command::new(editor).arg(temp_file_path).status()?;
        if !status.success() {
            exit(1);
        }
        let mut buffer = String::new();
        let mut source = File::open(temp_file_path)?;
        source.read_to_string(&mut buffer)?;

        let mut file = File::create(&self.path)?;
        writeln!(file, "{}", buffer)?;

        Ok(self)
    }
}
