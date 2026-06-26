use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;
use crate::{storage, dump, BazaR, Config};
use exn::ResultExt;

pub async fn push() -> BazaR<()> {
    let config = Config::get();
    let sync_config = config.sync.as_ref().ok_or_else(|| {
        crate::error::Error::Message("Sync not configured. Add [sync] section to baza.toml".into())
    })?;

    let data = storage::dump().await?;
    let dumped = dump::dump(&data, dump::Algorithm::Lz4)
        .or_raise(|| crate::error::Error::Message("Failed to dump database".into()))?;

    // Encrypt the dump before pushing to cloud
    let key = crate::key()?;
    let encrypted_dump = crate::encrypt_data(&dumped, &key)?;

    let bucket = get_bucket(sync_config)?;

    bucket.put_object("baza.dump", &encrypted_dump).await
        .map_err(|e| exn::Exn::new(crate::error::Error::Message(format!("Failed to upload to S3: {}", e))))?;

    println!("Database successfully pushed to cloud.");
    Ok(())
}

pub async fn pull() -> BazaR<()> {
    let config = Config::get();
    let sync_config = config.sync.as_ref().ok_or_else(|| {
        crate::error::Error::Message("Sync not configured. Add [sync] section to baza.toml".into())
    })?;

    let bucket = get_bucket(sync_config)?;

    let response = bucket.get_object("baza.dump").await
        .map_err(|e| exn::Exn::new(crate::error::Error::Message(format!("Failed to download from S3: {}", e))))?;

    let encrypted_data = response.bytes();

    // Decrypt the dump after pulling from cloud
    let key = crate::key()?;
    let decrypted_data = crate::decrypt_data(encrypted_data, &key)?;

    let restored = dump::restore::<Vec<(String, Vec<u8>)>>(&decrypted_data)
        .or_raise(|| crate::error::Error::Message("Failed to restore database from cloud dump".into()))?;

    storage::restore_unlocked(restored).await?;

    println!("Database successfully pulled from cloud.");
    Ok(())
}

fn get_bucket(sync: &crate::SyncConfig) -> BazaR<Box<Bucket>> {
    let credentials = Credentials::new(
        Some(&sync.access_key),
        Some(&sync.secret_key),
        None,
        None,
        None,
    ).map_err(|e| exn::Exn::new(crate::error::Error::Message(format!("Invalid S3 credentials: {}", e))))?;

    let region = Region::Custom {
        region: sync.region.clone(),
        endpoint: sync.endpoint.clone(),
    };

    Bucket::new(&sync.bucket, region, credentials)
        .map_err(|e| exn::Exn::new(crate::error::Error::Message(format!("Failed to initialize S3 bucket: {}", e))))
}
