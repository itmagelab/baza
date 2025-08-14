use std::{
    env,
    fs::{self, File},
};

use sequoia_openpgp::{self as openpgp, armor, serialize::Serialize};

use openpgp::cert::prelude::*;

use crate::{Config, DEFAULT_EMAIL};

fn keys_exists(name: &str) -> bool {
    ["cert", "key", "only_subkey", "revocation"]
        .iter()
        .all(|x| fs::exists(format!("{name}.{x}.pgp")).unwrap_or(false))
}

pub fn generate() -> openpgp::Result<()> {
    let config = Config::get();
    let dir = &config.main.datadir;
    let email = env::var("BAZA_EMAIL").unwrap_or(String::from(DEFAULT_EMAIL));
    let name = format!("{dir}/key");
    if keys_exists(&name) {
        return Ok(());
    }
    let (cert, revocation) = CertBuilder::new()
        .add_userid(email)
        .add_transport_encryption_subkey()
        .generate()?;
    let n = format!("{name}.cert.pgp");
    cert.armored().serialize(&mut File::create(n)?)?;
    let n = format!("{name}.key.pgp");
    cert.as_tsk().armored().serialize(&mut File::create(n)?)?;
    let n = format!("{name}.only_subkey.pgp");
    cert.as_tsk()
        .set_filter(|k| k.fingerprint() != cert.fingerprint())
        .emit_secret_key_stubs(true)
        .armored()
        .serialize(&mut File::create(n)?)?;
    let n = format!("{name}.revocation.pgp");
    let mut comments = cert.armor_headers();
    comments.insert(0, "Revocation certificate for the following key:".into());
    comments.insert(1, "".into());
    let mut w = armor::Writer::with_headers(
        File::create(n)?,
        armor::Kind::PublicKey,
        comments.iter().map(|c| ("Comment", c)),
    )?;
    openpgp::Packet::from(revocation).serialize(&mut w)?;
    w.finalize()?;

    Ok(())
}
