use std::fs;
use std::path::{Path, PathBuf};
use std::time::{UNIX_EPOCH};
use crate::config::Config;

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
        full_path.canonicalize().map_err(|e| FileError::FileError(format!("File Error: {}", e)))?;
        let normalized_path = normalize_path(&full_path);
        Ok(PathBuf::from(normalized_path))
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

pub fn normalize_path(path: &Path) -> String {
    let s = path.to_string_lossy();
    if s.starts_with(r"\\?\") {
        s[4..].replace("\\","/")
    }
    else {
        s.replace("\\", "/")
    }
}

pub fn create_forge_dirs(name: &str) -> std::io::Result<()> {
    let dir_path = Path::new("forge").join(name);
    if !dir_path.exists() {
        fs::create_dir_all(dir_path)?;
    }
    Ok(())   
}

pub fn find_r_paths(config: &Config) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = Vec::new();
    // only check if dependencies are set
    match &config.forge.dependencies {
        Some(deps) => {
            // check for all paths
            for path in &deps.library_paths {
                // check for all libraries
                for lib in &deps.libraries {
                    // check if .so exists or lib.so exists
                    let full_path = Path::new(path).join(format!("{}.so", lib));
                    let alt_full_path = Path::new(path).join(format!("lib{}.so", lib));
                    
                    if full_path.exists() && !paths.contains(&PathBuf::from(path)) {
                        let normalized_path = normalize_path(Path::new(path));
                        paths.push(PathBuf::from(normalized_path));
                        break;   
                    }
                    else if alt_full_path.exists() && !paths.contains(&PathBuf::from(path)) {
                        let normalized_path = normalize_path(Path::new(path));
                        paths.push(PathBuf::from(normalized_path));   
                        break;   
                    }
                }
            }
        }
        None => {}
    }
    paths
}