use globset::{Glob, GlobSetBuilder};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use std::collections::HashSet;
use crate::arguments::DiscoverOptions;
use anyhow::{bail, Result};
use crate::fs_utils::{add_to_build_toml, normalize_path};
use crate::fs_utils::BuildField::{IncludeDirs, Src};
use crate::ui::event_file_found;


pub fn discover(options: &DiscoverOptions, toml_path: PathBuf) -> Result<()> {
    if !toml_path.exists() {
        bail!("RustyForge.toml not found. Try `rustyforge init`, to initialize a new project.")
    }

    let c_files = find_c_files(".");
    let header_dirs = find_header_dirs(".");

    for c_file in c_files {
        let str = normalize_path(&c_file);
        if event_file_found(options, &str) {
            add_to_build_toml(&toml_path, Src, str.clone())?;
        }
    }
    for header_dir in header_dirs {
        let str = normalize_path(&header_dir);
        if event_file_found(options, &str) {
            add_to_build_toml(&toml_path, IncludeDirs, str.clone())?;
        }
    };
    Ok(())
}

/// checks if a file should be ignored by the discovery process
pub fn should_be_ignored(name: &String, ignore: &[String] ) -> bool {
    let mut builder = GlobSetBuilder::new();
    
    for pattern in ignore {
        if let Ok(glob) = Glob::new(pattern) {
            builder.add(glob);
        }
    }
    let globset = match builder.build() { 
        Ok(globset) => globset,
        Err(_) => return false,
    };
    
    globset.is_match(Path::new(name))
}

pub fn find_c_files(root: &str) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.file_type().is_file()
                && entry.path().extension().map_or(false, |ext| ext == "c")
        })
        .map(|entry| entry.into_path())
        .collect()
}
pub fn find_header_dirs(root: &str) -> Vec<PathBuf> {
    let mut dirs = HashSet::new();

    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file()
            && entry.path().extension().map_or(false, |ext| ext == "h")
        {
            if let Some(parent) = entry.path().parent() {
                dirs.insert(parent.to_path_buf());
            }
        }
    }

    dirs.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use tempfile::tempdir;
    use super::*;
    
    #[test]
    fn test_should_be_ignored(){
        let patterns = vec!["*.c".to_string(), "ignored/**".to_string()];
        assert!(should_be_ignored(&"main.c".to_string(), &patterns));
        assert!(should_be_ignored(&"ignored/file.h".to_string(), &patterns));
        assert!(!should_be_ignored(&"src/lib.h".to_string(), &patterns));
    }
    #[test]
    fn test_find_c_files_and_header_dirs() {
        let dir = tempdir().unwrap();
        let c_file_path = dir.path().join("main.c");
        let h_file_path = dir.path().join("main.h");
        std::fs::write(&c_file_path, "int main() {}").unwrap();
        std::fs::write(&h_file_path, "// header").unwrap();
        
        let c_files = find_c_files(dir.path().to_str().unwrap());
        assert_eq!(c_files.len(), 1);
        assert_eq!(c_files[0], c_file_path);
        let header_dirs = find_header_dirs(dir.path().to_str().unwrap());
        assert_eq!(header_dirs.len(), 1);
        assert_eq!(header_dirs[0], h_file_path.parent().unwrap());
    }

}














