//! Shared helpers for baza integration tests.
//!
//! Each test binary gets its own copy of this crate, so the mutex
//! serializes tests within a single binary. Cargo runs test binaries
//! sequentially, which keeps global state safe across binaries too.

use std::sync::{Mutex, OnceLock};

use baza_core::Config;

pub static TEST_MUTEX: Mutex<()> = Mutex::new(());

pub fn test_datadir() -> &'static str {
    static DIR: OnceLock<tempfile::TempDir> = OnceLock::new();
    DIR.get_or_init(|| tempfile::tempdir().expect("Failed to create tempdir"))
        .path()
        .to_str()
        .unwrap()
}

pub fn setup_test_env() {
    let test_dir = std::path::PathBuf::from(test_datadir());
    let _ = std::fs::remove_dir_all(&test_dir);
    std::fs::create_dir_all(&test_dir).expect("Failed to create test dir");

    let config_path = test_dir.join("baza.toml");
    let mut config = Config::default();
    config.main.datadir = test_dir.to_string_lossy().to_string();
    let config_str = toml::to_string(&config).expect("Failed to serialize config");
    std::fs::write(&config_path, config_str).expect("Failed to write config");
    Config::build(&config_path).expect("Failed to build config");
}
