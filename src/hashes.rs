use std::path::Path;
use sha2::{Sha256, Digest};
use std::fs::File;
use std::io::{BufReader, Read};
use serde::{Deserialize, Serialize};
use crate::fs_utils::{load_hash_cache_json, normalize_path, save_hash_cache_json};

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



pub fn get_cached_hash(filepath: &Path) -> Option<String> {
    let norm_path = normalize_path(filepath);
    let entries = load_hash_cache_json().expect("Failed to load hash cache");
    for entry in entries {
        if entry.path == norm_path {
            return Some(entry.hash);
        }
    }
    None
}

pub fn cache_hash(filepath: &Path, hash: String) {
    let mut entries = load_hash_cache_json().expect("Failed to load hash cache");
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
    save_hash_cache_json(&entries).expect("Failed to save hash cache");
}

