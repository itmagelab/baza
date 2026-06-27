use crate::{BazaR, Config};
use exn::ResultExt;
use s3::bucket::Bucket;
use s3::creds::Credentials;
use s3::region::Region;

/// Push database to S3 as a compressed dump
pub fn push() -> BazaR<()> {
    let config = Config::get();
    let s3_config = config.s3.as_ref().ok_or_else(|| {
        println!("S3 is not configured. Please add an [s3] section in baza.toml:");
        println!("\n[s3]");
        println!("endpoint = \"https://s3.amazonaws.com\"");
        println!("bucket = \"my-bucket\"");
        println!("region = \"us-east-1\"");
        println!("access_key_id = \"YOUR_ACCESS_KEY\"");
        println!("secret_access_key = \"YOUR_SECRET_KEY\"");
        println!("key = \"db.redb\" # optional");
        println!("path_style = true # optional, set true for MinIO/R2\n");
        exn::Exn::new(crate::error::Error::Message(
            "S3 configuration is missing".into(),
        ))
    })?;

    let access_key = s3_config
        .access_key_id
        .clone()
        .or_else(|| std::env::var("AWS_ACCESS_KEY_ID").ok())
        .or_else(|| std::env::var("BAZA_S3_ACCESS_KEY_ID").ok())
        .ok_or_else(|| {
            exn::Exn::new(crate::error::Error::Message(
                "S3 access key ID is missing. Set it in config or BAZA_S3_ACCESS_KEY_ID env var."
                    .into(),
            ))
        })?;

    let secret_key = s3_config
        .secret_access_key
        .clone()
        .or_else(|| std::env::var("AWS_SECRET_ACCESS_KEY").ok())
        .or_else(|| std::env::var("BAZA_S3_SECRET_ACCESS_KEY").ok())
        .ok_or_else(|| {
            exn::Exn::new(crate::error::Error::Message(
                "S3 secret access key is missing. Set it in config or BAZA_S3_SECRET_ACCESS_KEY env var.".into()
            ))
        })?;

    let key_name = s3_config.key.as_deref().unwrap_or("db.redb");

    let region = Region::Custom {
        region: s3_config.region.clone(),
        endpoint: s3_config.endpoint.clone(),
    };

    let credentials = Credentials::new(Some(&access_key), Some(&secret_key), None, None, None)
        .or_raise(|| crate::error::Error::Message("Failed to create S3 credentials".into()))?;

    let mut bucket = Bucket::new(&s3_config.bucket, region, credentials)
        .or_raise(|| crate::error::Error::Message("Failed to initialize S3 bucket".into()))?;

    if s3_config.path_style.unwrap_or(true) {
        bucket = bucket.with_path_style();
    }

    println!("Creating local database dump...");
    let data = pollster::block_on(crate::storage::dump())?;
    let dumped = crate::dump::dump(&data, crate::dump::Algorithm::Lz4)
        .or_raise(|| crate::error::Error::Message("Failed to dump database".into()))?;

    println!("Uploading database dump to S3 (key: {})...", key_name);
    bucket
        .put_object(key_name, &dumped)
        .or_raise(|| crate::error::Error::Message("Failed to upload database dump to S3".into()))?;

    println!("Upload completed successfully.");
    Ok(())
}

/// Pull database from S3 and restore local storage
pub fn pull() -> BazaR<()> {
    let config = Config::get();
    let s3_config = config.s3.as_ref().ok_or_else(|| {
        println!("S3 is not configured. Please add an [s3] section in baza.toml:");
        println!("\n[s3]");
        println!("endpoint = \"https://s3.amazonaws.com\"");
        println!("bucket = \"my-bucket\"");
        println!("region = \"us-east-1\"");
        println!("access_key_id = \"YOUR_ACCESS_KEY\"");
        println!("secret_access_key = \"YOUR_SECRET_KEY\"");
        println!("key = \"db.redb\" # optional");
        println!("path_style = true # optional, set true for MinIO/R2\n");
        exn::Exn::new(crate::error::Error::Message(
            "S3 configuration is missing".into(),
        ))
    })?;

    let access_key = s3_config
        .access_key_id
        .clone()
        .or_else(|| std::env::var("AWS_ACCESS_KEY_ID").ok())
        .or_else(|| std::env::var("BAZA_S3_ACCESS_KEY_ID").ok())
        .ok_or_else(|| {
            exn::Exn::new(crate::error::Error::Message(
                "S3 access key ID is missing. Set it in config or BAZA_S3_ACCESS_KEY_ID env var."
                    .into(),
            ))
        })?;

    let secret_key = s3_config
        .secret_access_key
        .clone()
        .or_else(|| std::env::var("AWS_SECRET_ACCESS_KEY").ok())
        .or_else(|| std::env::var("BAZA_S3_SECRET_ACCESS_KEY").ok())
        .ok_or_else(|| {
            exn::Exn::new(crate::error::Error::Message(
                "S3 secret access key is missing. Set it in config or BAZA_S3_SECRET_ACCESS_KEY env var.".into()
            ))
        })?;

    let key_name = s3_config.key.as_deref().unwrap_or("db.redb");

    let region = Region::Custom {
        region: s3_config.region.clone(),
        endpoint: s3_config.endpoint.clone(),
    };

    let credentials = Credentials::new(Some(&access_key), Some(&secret_key), None, None, None)
        .or_raise(|| crate::error::Error::Message("Failed to create S3 credentials".into()))?;

    let mut bucket = Bucket::new(&s3_config.bucket, region, credentials)
        .or_raise(|| crate::error::Error::Message("Failed to initialize S3 bucket".into()))?;

    if s3_config.path_style.unwrap_or(true) {
        bucket = bucket.with_path_style();
    }

    println!("Downloading database dump from S3...");
    let response = bucket.get_object(key_name).or_raise(|| {
        crate::error::Error::Message("Failed to download database dump from S3".into())
    })?;

    println!("Restoring database from dump...");
    let restored = crate::dump::restore::<Vec<(String, Vec<u8>)>>(response.as_slice())
        .or_raise(|| crate::error::Error::Message("Failed to parse remote database dump".into()))?;

    let is_init = pollster::block_on(crate::storage::is_initialized())?;
    if !is_init {
        println!("Local database not initialized. Initializing...");
        crate::storage::initialize()?;
    }

    pollster::block_on(crate::storage::restore(restored))?;

    println!("Database restored successfully.");
    Ok(())
}
