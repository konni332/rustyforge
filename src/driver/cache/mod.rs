use serde::{Serialize, de::DeserializeOwned};
use std::{
    hash::Hash,
    path::{Path, PathBuf},
    sync::RwLock,
};

use crate::{CoreResult, warn};

mod build;
pub use build::BuildCache;

const DEFAULT_CACHE_VERSION: u64 = 1;

pub struct CacheFile<I>
where
    I: CacheFileInner + Serialize + DeserializeOwned,
{
    path: PathBuf,
    inner: RwLock<I>,
}

impl<I: CacheFileInner + Serialize + DeserializeOwned> CacheFile<I> {
    pub fn new<P: AsRef<Path>>(path: P) -> CoreResult<Self> {
        let cache = if path.as_ref().exists() {
            let src = std::fs::read(&path)?;
            postcard::from_bytes(&src)?
        } else {
            I::new()
        };
        let inner = RwLock::new(cache);
        Ok(Self {
            inner,
            path: path.as_ref().to_path_buf(),
        })
    }
    pub fn insert(&self, k: &I::Key, v: I::Value) -> Option<I::Value> {
        let mut guard = self.inner.write().unwrap();
        guard.insert(k, v)
    }
    pub fn contains(&self, k: &I::Key) -> bool {
        self.inner.read().unwrap().contains(k)
    }
    pub fn get(&self, k: &I::Key) -> Option<I::Value> {
        let guard = self.inner.read().unwrap();
        guard.get(k).cloned()
    }
    pub fn flush(&self) -> CoreResult<()> {
        let guard = self.inner.read().unwrap();
        let src: Vec<u8> = postcard::to_allocvec(&*guard)?;
        std::fs::write(&self.path, src)?;
        Ok(())
    }
    pub fn seed(&self) -> u64 {
        self.inner.read().unwrap().seed()
    }
}

impl<I: CacheFileInner + Serialize + DeserializeOwned> Drop for CacheFile<I> {
    fn drop(&mut self) {
        if let Err(e) = self.flush() {
            let path = &self.path;
            warn!(&format!(
                "Failed to flush cache to file ({}) while dropping: {e}\n",
                path.display(),
            ));
        }
    }
}

pub trait CacheFileInner {
    type Key: Hash + Serialize + DeserializeOwned + Send + Sync;
    type Value: Serialize + DeserializeOwned + Send + Sync + Clone;

    fn new() -> Self
    where
        Self: Sized;
    fn insert(&mut self, k: &Self::Key, v: Self::Value) -> Option<Self::Value>;
    fn get(&self, k: &Self::Key) -> Option<&Self::Value>;
    fn seed(&self) -> u64;
    fn contains(&self, k: &Self::Key) -> bool;
}
