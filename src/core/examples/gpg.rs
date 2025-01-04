use openpgp::cert::Cert;
use openpgp::serialize::stream::Encryptor2;
use sequoia_openpgp as openpgp;
use sequoia_openpgp::crypto::Decryptor;
use sequoia_openpgp::parse::Parse;
use sequoia_openpgp::policy::StandardPolicy;
use sequoia_openpgp::serialize::stream::{LiteralWriter, Message, Recipient};
use std::fs::File;
use std::io::{BufReader, Read, Write};

fn encrypt_with_pgp(
    input_path: &str,
    output_path: &str,
    public_key_path: &str,
) -> openpgp::Result<()> {
    let public_key_file = File::open(public_key_path)?;
    let cert = Cert::from_reader(BufReader::new(public_key_file))?;

    let p = &StandardPolicy::new();

    let recipients = cert
        .keys()
        .with_policy(p, None)
        .supported()
        .alive()
        .revoked(false)
        .for_transport_encryption()
        .map(|ka| Recipient::new(ka.key().keyid(), ka.key()));

    let mut sink = File::create(output_path)?;
    let message = Message::new(&mut sink);
    let message = Encryptor2::for_recipients(message, recipients).build()?;

    let mut input_file = File::open(input_path)?;
    let mut buffer = Vec::new();
    input_file.read_to_end(&mut buffer)?;

    let mut message = LiteralWriter::new(message).build()?;

    message.write_all(&buffer)?;
    message.finalize()?;

    Ok(())
}

fn decrypt_file(
    input_path: &str,
    output_path: &str,
    private_key_path: &str,
) -> openpgp::Result<()> {
    use openpgp::crypto::SessionKey;
    use openpgp::parse::{stream::*, Parse};
    use openpgp::types::SymmetricAlgorithm;
    use openpgp::{
        packet::{PKESK, SKESK},
        Cert, Result,
    };
    use sequoia_openpgp as openpgp;
    use sequoia_openpgp::policy::StandardPolicy;
    use std::io::Read;

    let p = &StandardPolicy::new();

    struct Helper;
    impl VerificationHelper for Helper {
        fn get_certs(&mut self, _ids: &[openpgp::KeyHandle]) -> Result<Vec<Cert>> {
            Ok(Vec::new())
        }
        fn check(&mut self, _structure: MessageStructure) -> Result<()> {
            Ok(())
        }
    }
    impl DecryptionHelper for Helper {
        fn decrypt<D>(
            &mut self,
            _: &[PKESK],
            skesks: &[SKESK],
            _sym_algo: Option<SymmetricAlgorithm>,
            mut decrypt: D,
        ) -> Result<Option<openpgp::Fingerprint>>
        where
            D: FnMut(SymmetricAlgorithm, &SessionKey) -> bool,
        {
            let _ = skesks[0]
                .decrypt(&"baza".into())
                .map(|(algo, session_key)| decrypt(algo, &session_key));
            Ok(None)
        }
    }

    let mut input_file = File::open(input_path)?;
    let mut message = Vec::new();
    input_file.read_to_end(&mut message)?;

    let h = Helper {};
    let mut v = DecryptorBuilder::from_bytes(&message)?.with_policy(p, None, h)?;

    let mut buffer = Vec::new();
    v.read_to_end(&mut buffer)?;

    let mut file = File::create(output_path)?;
    file.write_all(&buffer)?;

    Ok(())
}

fn main() -> openpgp::Result<()> {
    let input_path = "/var/tmp/baza/work/selectel/bspb/avsemenov02";
    let output_path = "example.txt.gpg";
    let public_key_path = "public_key.asc";

    encrypt_with_pgp(input_path, output_path, public_key_path)?;
    decrypt_file("example.txt.gpg", "example.txt", public_key_path)?;
    println!("Файл успешно зашифрован с использованием PGP!");
    Ok(())
}
