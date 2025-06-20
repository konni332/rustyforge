use std::path::{Path, PathBuf};
use sha2::{Sha256, Digest};
use std::fs::File;
use std::io::{BufReader, Read};
use serde::{Deserialize, Serialize};
use crate::fs_utils::{load_hash_cache_json, normalize_path, save_hash_cache_json, std_hash_cache_path};
use anyhow::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct HashCache {
    pub path: String,
    pub hash: String,
}

pub fn hash(file_path: &Path) -> Result<String, std::io::Error> {
    if !file_path.exists() || !file_path.is_file() {
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "File not found"));
    }
    
    let mut reader = BufReader::new(File::open(file_path)?);
    let mut hasher = Sha256::new();
    
    let mut buffer = [0u8; 4096];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}



pub fn get_cached_hash(filepath: &Path, json_path: PathBuf) -> Option<String> {
    let norm_path = normalize_path(filepath);
    let entries = load_hash_cache_json(json_path).expect("Failed to load hash cache");
    
    for entry in entries {
        if entry.path == norm_path {
            return Some(entry.hash);
        }
    }
    None
}

pub fn cache_hash(filepath: &Path, json_path: PathBuf) -> Result<()> {
    let hash = hash(filepath)?;
    
    let mut entries = load_hash_cache_json(std_hash_cache_path()?)
        .expect("Failed to load hash cache");
    
    let norm_path = normalize_path(filepath);
    
    let mut found = false;
    for entry in &mut entries {
        if entry.path == norm_path {
            entry.hash = hash.clone();
            found = true;
            break;
        }
    }
    if !found {
        entries.push(HashCache {
            path: norm_path,
            hash,
        });
    }
    save_hash_cache_json(&entries, json_path).expect("Failed to save hash cache");
    Ok(())
}

pub fn file_changed(filepath: &Path, json_path: PathBuf) -> Result<bool> {
    let new_hash = hash(filepath)?;
    let cached_hash = get_cached_hash(filepath, json_path);
    if cached_hash.is_none() {
        return Ok(true);
    }
    Ok(cached_hash.unwrap() != new_hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs::{write};
    #[test]
    fn test_cache_hash_and_get_cached_hash_and_file_changed() {
        let dir = tempdir().unwrap();
        let json_path = dir.path().join("hash_cache.json");
        let file_path = dir.path().join("test.txt");

        write(&file_path, b"hello world").unwrap();

        cache_hash(&file_path, json_path.clone()).expect("Failed to cache hash");

        let hash = get_cached_hash(&file_path, json_path.clone());
        assert!(hash.is_some());

        let changed = file_changed(&file_path, json_path.clone()).unwrap();
        assert!(!changed);

        write(&file_path, b"changed").unwrap();

        let changed = file_changed(&file_path, json_path.clone()).unwrap();
        assert!(changed);
    }
}
