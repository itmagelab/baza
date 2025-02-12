use std::{
    env,
    fs::{self, File},
};

use sequoia_openpgp::{self as openpgp, armor, serialize::Serialize};

use openpgp::cert::prelude::*;

use crate::{config, DEFAULT_EMAIL};

fn keys_exists(name: &str) -> bool {
    ["cert", "key", "only_subkey", "revocation"]
        .iter()
        .all(|x| fs::exists(format!("{}.{}.pgp", name, x)).unwrap_or(false))
}

pub fn generate() -> openpgp::Result<()> {
    let dir = config().main.datadir;
    let email = env::var("BAZA_EMAIL").unwrap_or(String::from(DEFAULT_EMAIL));
    let name = format!("{}/key", dir);
    if keys_exists(&name) {
        return Ok(());
    }
    let (cert, revocation) = CertBuilder::new()
        .add_userid(email)
        .add_transport_encryption_subkey()
        .generate()?;
    let n = format!("{}.cert.pgp", name);
    cert.armored().serialize(&mut File::create(n)?)?;
    let n = format!("{}.key.pgp", name);
    cert.as_tsk().armored().serialize(&mut File::create(n)?)?;
    let n = format!("{}.only_subkey.pgp", name);
    cert.as_tsk()
        .set_filter(|k| k.fingerprint() != cert.fingerprint())
        .emit_secret_key_stubs(true)
        .armored()
        .serialize(&mut File::create(n)?)?;
    let n = format!("{}.revocation.pgp", name);
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
