use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn create_forge_dir() -> std::io::Result<()> {
    let dir_path = Path::new("forge");
    if !dir_path.exists() {
        fs::create_dir_all(dir_path)?;
    }
    Ok(())
}

#[derive(Debug, PartialEq)]
pub enum FileError {
    FileNotFound(String),
    FileError(String),
    CwdError(String),
}

pub fn find_file(filename: &str) -> Result<PathBuf, FileError> {
    let cwd = std::env::current_dir().map_err(|e| FileError::CwdError(format!("CWD Error: {}", e)))?;
    let full_path = cwd.join(filename);
    if full_path.exists() && full_path.is_file() {
        Ok(full_path.canonicalize().map_err(|e| FileError::FileError(format!("File Error: {}", e)))?)
    } else {
        Err(FileError::FileNotFound(format!("File not found: {}", filename)))
    }
}

pub fn get_timestamp(absolut_path: PathBuf) -> Result<u64, String> {
    let metadata = fs::metadata(absolut_path).map_err(|e| format!("File Error: {}", e))?;
    let modified_time = metadata.modified().map_err(|e| format!("File Error: {}", e))?;
    
    let duration = modified_time.duration_since(UNIX_EPOCH).map_err(|e| format!("File Error: {}", e))?;
    Ok(duration.as_secs())
}

pub fn get_equivalent_forge_path(input_path: &Path) -> Result<PathBuf, String> {
    let cwd = std::env::current_dir().map_err(|e| format!("CWD Error: {}", e))?;
    let file_stem = input_path.file_stem().and_then(|s| s.to_str())
        .ok_or("File stem not valid UTF-8")?;
    let forge_path = cwd.join("forge").join(format!("{}.o", file_stem));
    Ok(forge_path)
}


