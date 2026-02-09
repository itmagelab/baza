#[cfg(not(target_arch = "wasm32"))]
pub mod redb;
#[cfg(target_arch = "wasm32")]
pub mod web;

#[cfg(not(target_arch = "wasm32"))]
use crate::Config;
use crate::{bundle::Bundle, BazaR};
use async_trait::async_trait;

#[cfg(not(target_arch = "wasm32"))]
pub fn storage_dir(dir: &'static str) -> std::path::PathBuf {
    std::path::PathBuf::from(format!("{}/data/{}", &Config::get().main.datadir, dir))
}

#[async_trait(?Send)]
#[cfg(not(target_arch = "wasm32"))]
pub(crate) trait StorageBackend: Sync + Send {
    async fn create(&self, bundle: Bundle, replace: bool) -> BazaR<()>;
    async fn read(&self, bundle: Bundle) -> BazaR<()>;
    async fn update(&self, bundle: Bundle) -> BazaR<()>;
    async fn delete(&self, bundle: Bundle) -> BazaR<()>;
    async fn search(&self, pattern: String) -> BazaR<()>;
    async fn copy_to_clipboard(&self, bundle: Bundle, ttl: u64) -> BazaR<()>;
}

#[async_trait(?Send)]
#[cfg(target_arch = "wasm32")]
pub(crate) trait StorageBackend {
    async fn create(&self, bundle: Bundle, replace: bool) -> BazaR<()>;
    async fn read(&self, bundle: Bundle) -> BazaR<()>;
    async fn update(&self, bundle: Bundle) -> BazaR<()>;
    async fn delete(&self, bundle: Bundle) -> BazaR<()>;
    async fn search(&self, pattern: String) -> BazaR<()>;
    async fn copy_to_clipboard(&self, bundle: Bundle, ttl: u64) -> BazaR<()>;
}

pub(crate) async fn with_backend<F, Fut, R>(f: F) -> BazaR<R>
where
    F: FnOnce(&'static dyn StorageBackend) -> Fut,
    Fut: std::future::Future<Output = BazaR<R>>,
{
    #[cfg(not(target_arch = "wasm32"))]
    {
        let backend = self::redb::Redb::instance()?;
        f(backend).await
    }

    #[cfg(target_arch = "wasm32")]
    {
        let backend = self::web::WebStorage::instance().await?;
        f(backend).await
    }
}

pub fn initialize() -> BazaR<()> {
    #[cfg(not(target_arch = "wasm32"))]
    self::redb::initialize()?;
    Ok(())
}

pub(crate) async fn create(bundle: Bundle) -> BazaR<()> {
    with_backend(|backend| backend.create(bundle, true)).await
}

pub(crate) async fn read(bundle: Bundle) -> BazaR<()> {
    with_backend(|backend| backend.read(bundle)).await
}

pub(crate) async fn update(bundle: Bundle) -> BazaR<()> {
    with_backend(|backend| backend.update(bundle)).await
}

pub(crate) async fn delete(bundle: Bundle) -> BazaR<()> {
    with_backend(|backend| backend.delete(bundle)).await
}

pub async fn search(str: String) -> BazaR<()> {
    with_backend(|backend| backend.search(str)).await
}

pub(crate) async fn copy_to_clipboard(bundle: Bundle, ttl: u64) -> BazaR<()> {
    with_backend(|backend| backend.copy_to_clipboard(bundle, ttl)).await
}
