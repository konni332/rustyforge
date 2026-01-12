use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

use crate::driver::cache::{CacheFileInner, DEFAULT_CACHE_VERSION};

const BUILD_CACHE_SEED: u64 = 0xBCACE ^ DEFAULT_CACHE_VERSION;

/// Represents the build Cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildCache {
    data: HashMap<u64, PathBuf>,
    seed: u64,
}

impl CacheFileInner for BuildCache {
    type Value = PathBuf;
    type Key = u64;

    fn get(&self, k: &Self::Key) -> Option<&Self::Value> {
        self.data.get(k)
    }

    fn new() -> Self
    where
        Self: Sized,
    {
        Self {
            data: HashMap::new(),
            seed: BUILD_CACHE_SEED,
        }
    }

    fn insert(&mut self, k: &Self::Key, v: Self::Value) -> Option<Self::Value> {
        self.data.insert(*k, v)
    }
    fn seed(&self) -> u64 {
        self.seed
    }
    fn contains(&self, k: &Self::Key) -> bool {
        self.data.contains_key(k)
    }
}
