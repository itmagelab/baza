use crate::prelude::*;

pub enum MessageType {
    Clean,
    Data,
    Info,
    Warning,
    Error,
}

pub fn m(msg: &str, _type: MessageType) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use colored::Colorize;

        let colored_msg = match _type {
            MessageType::Clean => msg.to_string(),
            MessageType::Data => format!("{}", msg.bright_blue()),
            MessageType::Info => format!("{}", msg.bright_green()),
            MessageType::Warning => format!("{}", msg.bright_yellow()),
            MessageType::Error => format!("{}", msg.bright_red()),
        };
        println!("{colored_msg}");
    }

    #[cfg(target_arch = "wasm32")]
    let msg = msg; // No coloring for WASM log for now

    tracing::info!("{msg}");
}

// TODO: Make with NamedTmpFolder
/// Cleanup temporary files
#[cfg(not(target_arch = "wasm32"))]
pub fn cleanup_tmp_folder() -> BazaR<()> {
    let datadir = &Config::get().main.datadir;
    let tmpdir = format!("{datadir}/tmp");
    if std::fs::remove_dir_all(&tmpdir).is_err() {
        tracing::debug!("Tmp folder already cleaned");
    };
    std::fs::create_dir_all(format!("{datadir}/tmp"))
        .or_raise(|| error::Error::Message("Failed to create tmp directory".into()))?;
    Ok(())
}

#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub(crate) fn as_hash(str: &str) -> [u8; 32] {
    let mut hasher = sha2::Sha256::new();
    hasher.update(str.as_bytes());
    let result = hasher.finalize();
    result.into()
}
