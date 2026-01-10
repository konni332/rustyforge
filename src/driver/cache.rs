use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::hash::Hasher;
use std::{
    collections::HashMap,
    hash::Hash,
    path::{Path, PathBuf},
};

use crate::{CoreResult, internal_error};

const DEFAULT_CACHE_VERSION: u64 = 1;
const DEFAULT_CACHE_SEED: u64 = 0xCACE_BEEF ^ DEFAULT_CACHE_VERSION;

/// Represents a cache file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "V: Deserialize<'de>"))]
pub struct Cache<V: Serialize> {
    path: PathBuf,
    data: HashMap<u64, V>,
    seed: u64,
}

impl<V: DeserializeOwned + Serialize> Cache<V> {
    /// Read from path and deserialize the data.
    pub fn new<P: AsRef<Path>>(path: P) -> CoreResult<Self> {
        Self::with_seed(path, DEFAULT_CACHE_SEED)
    }
    pub fn with_seed<P: AsRef<Path>>(path: P, seed: u64) -> CoreResult<Self> {
        let src = std::fs::read(&path)?;
        let data: HashMap<u64, V> = postcard::from_bytes(&src)
            .map_err(|err| {
                internal_error!("Failed to deserialize cache: {}", err);
            })
            .unwrap();

        // Remove the cache file. This is due to possible partial writes or failed writes of the
        // cache when dropping. To be save we'd rather redo all the work than using poisoned cache
        std::fs::remove_file(&path)?;
        Ok(Self {
            path: path.as_ref().to_path_buf(),
            data,
            seed: seed,
        })
    }
    /// Insert key, value pair into the cache. The key will be hashed using twox_hash::XxHash64
    pub fn insert<K: Hash>(&mut self, k: &K, v: V) -> Option<V> {
        let mut hasher = twox_hash::XxHash64::with_seed(self.seed);

        k.hash(&mut hasher);
        self.data.insert(hasher.finish(), v)
    }
    /// Get key, from the cache. The key will be hashed using twox_hash::XxHash64
    pub fn get<K: Hash>(&mut self, k: &K) -> Option<&V> {
        let mut hasher = twox_hash::XxHash64::with_seed(self.seed);
        k.hash(&mut hasher);
        self.data.get(&hasher.finish())
    }
    /// Insert key, value pair into the cache. The key should already be hashed
    pub fn insert_hash(&mut self, k: u64, v: V) -> Option<V> {
        self.data.insert(k, v)
    }
    /// Get key, from the cache. The key should already be hashed
    pub fn get_hash(&mut self, k: &u64) -> Option<&V> {
        self.data.get(k)
    }
}

impl<V: Serialize> Drop for Cache<V> {
    fn drop(&mut self) {
        let contents = match postcard::to_allocvec(&self.data) {
            Ok(c) => c,
            Err(e) => {
                internal_error!("Failed to serialize cache contents: {}", e);
            }
        };
        std::fs::write(&self.path, contents).ok();
    }
}
