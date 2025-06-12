use globset::{Glob, GlobSetBuilder};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use std::collections::HashSet;
use crate::arguments::DiscoverOptions;
use anyhow::{Result};
use crate::fs_utils::normalize_path;

pub fn discover(options: &DiscoverOptions) -> Result<()> {
    let c_files = find_c_files(".");
    let header_dirs = find_header_dirs(".");
    
    for c_file in c_files {
        let str = normalize_path(&c_file);
        if !should_be_ignored(&str, options.ignore.as_slice()) {
            println!("{}", str);
        }
        
    }
    for header_dir in header_dirs {
        let str = normalize_path(&header_dir);
        if !should_be_ignored(&str, options.ignore.as_slice()) {
            println!("{}", str);
        }
    };
    Ok(())
}

/// checks if a file should be ignored by the discovery process
fn should_be_ignored(name: &String, ignore: &[String] ) -> bool {
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
