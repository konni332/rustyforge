use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use crate::config::{Build, Config, Forge, Project};
use crate::hashes::HashCache;
use anyhow::{Result, bail};

pub fn create_forge_dir() -> Result<()> {
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

pub fn get_equivalent_forge_path(input_path: &Path, config: &Config, shared: bool) -> Result<PathBuf, String> {
    let cwd = std::env::current_dir().map_err(|e| format!("CWD Error: {}", e))?;
    let file_stem = input_path.file_stem().and_then(|s| s.to_str())
        .ok_or("File stem not valid UTF-8")?;
    
    let forge_path: PathBuf;
    if shared { 
        forge_path = cwd.join("forge").join("libs").join("obj").join(format!("{}.o", file_stem));
    }
    else if config.args.debug {
        forge_path = cwd.join("forge").join("debug").join(format!("{}.o", file_stem));
    }
    else {
        forge_path = cwd.join("forge").join("release").join(format!("{}.o", file_stem));
    }
    Ok(forge_path)
}

pub fn normalize_path(path: &Path) -> String {
    let path = path.strip_prefix("./").unwrap_or(path);
    let s = path.to_string_lossy();
    if s.starts_with(r"\\?\") {
        s[4..].replace("\\", "/")
    } else {
        s.replace("\\", "/")
    }
}


pub fn create_forge_sub_dir(name: &str) -> Result<()> {
    let dir_path = Path::new("forge").join(name);
    if !dir_path.exists() {
        fs::create_dir_all(dir_path)?;
    }
    Ok(())   
}

// allow dead code because this is only used on linux and macOS
#[allow(dead_code)]
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

pub fn std_toml_path() -> Result<PathBuf> {
    Ok(std::env::current_dir()?
        .join("RustyForge.toml"))
}

pub fn std_hash_cache_path() -> Result<PathBuf> {
    Ok(std::env::current_dir()?
        .join("forge")
        .join(".forge")
        .join("hash_cache.json"))
}

pub fn init_hash_cache_json(json_path: PathBuf) -> Result<()> {
    if !json_path.exists() {
        if let Some(parent) = json_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&json_path, "[]")?;
    }

    Ok(())
}


pub fn load_hash_cache_json(json_path: PathBuf) -> Result<Vec<HashCache>, Box<dyn std::error::Error>> {
    let data = if Path::new(&json_path).exists() {
        fs::read_to_string(json_path)?
    }
    else { 
        "[]".to_string()
    };
    
    let entries: Vec<HashCache> = serde_json::from_str(&data)?;
    
    Ok(entries)  
}

pub fn save_hash_cache_json(entries: &Vec<HashCache>, json_path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let data = serde_json::to_string_pretty(entries)?;
    fs::write(json_path, data)?;
    Ok(())
}
pub fn init_default_toml() -> Result<()> {
    let cwd = std::env::current_dir()?;

    let dir_name = cwd
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    let default = Forge {
        project: Project {
            name: dir_name.to_string(),
            targets: vec!["bin".to_string()],
        },
        build: Build {
            compiler: Some("gcc".to_string()),
            src: vec![],
            include_dirs: vec![],
            output: dir_name.to_string(),
            cflags: None,
            ldflags: None,
        },
        dependencies: None,
    };

    let toml_string = toml::to_string_pretty(&default)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    let path = cwd.join("RustyForge.toml");

    if path.exists() {
        fs::remove_file(&path)?; 
    }

    let mut file = fs::File::create(&path)?;
    file.write_all(toml_string.as_bytes())?;
    file.sync_all()?; 

    println!("Created default RustyForge.toml at {}", path.display());
    Ok(())
}


pub fn ensure_necessary_files() -> Result<()> {
    let cwd = std::env::current_dir()?; // current working directory
    let forge_dir = cwd.join("forge"); // general forge directory
    let forge_dir_hidden = forge_dir.join(".forge"); // hidden forge directory
    let hash_cache_path = forge_dir_hidden.join("hash_cache.json");
    let toml_path = cwd.join("RustyForge.toml");
    
    let required_paths = [
        &forge_dir,
        &forge_dir_hidden,
        &hash_cache_path,
        &toml_path,
    ];
    // check if all required files exist
    if let Some(missing_path) = required_paths.iter().find(|p| !p.exists()) {
        bail!(
            "Missing required file: {}\n\nPlease run `rustyforge init` to initialize the forge directory.",
            missing_path.display()
        )
    }
    Ok(())
}

pub fn init_forge_structure() -> Result<()> {
    // create forge files
    create_forge_dir()?;
    create_forge_sub_dir(".forge")?;
    init_hash_cache_json(std_hash_cache_path()?)?;
    init_default_toml()?;
    // create the default project structure
    fs::create_dir_all("src")?;
    fs::create_dir_all("include")?;
    Ok(())  
}

pub enum BuildField {
    Src,
    IncludeDirs,
}

pub fn add_to_build_toml(toml_path: &Path, field: BuildField, value: String) -> Result<()> {
    
    let contents = fs::read_to_string(&toml_path)?;
    let mut forge = toml::from_str::<Forge>(&contents)?;
    let vec_ref = match field {
        BuildField::Src => &mut forge.build.src,
        BuildField::IncludeDirs => &mut forge.build.include_dirs,
    };
    
    if !vec_ref.contains(&value) {
        vec_ref.push(value);
    }
    let updated = toml::to_string_pretty(&forge)?;
    fs::write(&toml_path, updated)?;
    Ok(()) 
}

pub fn find_o_files(rel_path: &Path) -> Vec<PathBuf>{
    let cwd = std::env::current_dir().expect("Failed to get current directory");
    let cwd = cwd.join(rel_path);

    let mut o_files = Vec::new();

    for entry in fs::read_dir(cwd).expect("Failed to read forge/ directory") {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "o" {
                    let norm_abs_path = find_file(path.to_str().unwrap()).unwrap();
                    o_files.push(norm_abs_path);
                }
            }
        }
    }
    o_files
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{File};
    use std::io::Write;
    use tempfile::tempdir;
    use std::path::Path;
    use std::path::PathBuf;
    use crate::tests::dummy_config;
    #[test]
    fn test_find_file_exists() {
        let dir = tempdir().expect("Failed to create temp dir");
        let file_path = dir.path().join("test.txt");
        
        let mut file = File::create(&file_path).expect("Failed to create file");
        writeln!(file, "Hello, world!").expect("Failed to write to file");
        
        let orig_cwd = std::env::current_dir().expect("Failed to get current directory");
        std::env::set_current_dir(dir.path()).expect("Failed to set current directory");
        
        let result = find_file("test.txt");
        assert!(result.is_ok());
        let found_path = result.unwrap();
        assert_eq!(found_path, file_path);
        
        std::env::set_current_dir(orig_cwd).expect("Failed to reset current directory");
    }
    #[test]
    fn test_find_file_not_exists() {
         let orig_cwd = std::env::current_dir().unwrap();

        let result = find_file("file_that_does_not_exist.txt");
        assert!(result.is_err());
        if let Err(FileError::FileNotFound(msg)) = result {
            assert!(msg.contains("file_that_does_not_exist.txt"));
        } else {
            panic!("Expected FileNotFound error");
        }

        std::env::set_current_dir(orig_cwd).expect("Failed to reset CWD");
    }
    
    #[test]
    fn test_find_o_files(){
        let dir = tempdir().expect("Failed to create temp dir");
        
        let file1 = dir.path().join("file1.o");
        let file2 = dir.path().join("file2.o");
        
        File::create(&file1).expect("Failed to create file1");
        File::create(&file2).expect("Failed to create file2");
        
        let other_file = dir.path().join("other_file.txt");
        File::create(&other_file).expect("Failed to create other_file");
        
        let orig_cwd = std::env::current_dir().expect("Failed to get current directory");
        std::env::set_current_dir(dir.path()).expect("Failed to set current directory");
        
        let result = find_o_files(Path::new("."));
        assert_eq!(result.len(), 2);
        assert!(result.contains(&PathBuf::from(normalize_path(&file1))));
        assert!(result.contains(&PathBuf::from(normalize_path(&file2))));
        assert!(!result.contains(&PathBuf::from(normalize_path(&other_file))));
        
        std::env::set_current_dir(orig_cwd).expect("Failed to reset CWD");
    }
    #[test]
    #[should_panic(expected = "Failed to read forge/ directory")]
    fn test_find_o_files_missing_dir(){
        find_o_files(Path::new("missing_dir"));
    }
    
    #[test]
    fn test_normalize_path_strip_dot_slash() {
        let path = Path::new("./test.txt");
        let normalized = normalize_path(path);
        assert_eq!(normalized, "test.txt");
    }
    #[test]
    fn test_normalize_path_windows_unc_prefix() {
        let path = Path::new(r"\\?\C:\Users\test.txt");
        let normalized = normalize_path(path);
        assert_eq!(normalized, "C:/Users/test.txt");
    }
    #[test]
    fn test_normalize_path_backslashes() {
        let path = Path::new(r"C:\folder\subfolder\file.o");
        let normalized = normalize_path(path);
        assert_eq!(normalized, "C:/folder/subfolder/file.o");
    }

    #[test]
    fn test_normalize_path_no_prefix() {
        let path = Path::new("folder/subfolder/file.o");
        let normalized = normalize_path(path);
        assert_eq!(normalized, "folder/subfolder/file.o");
    }
    
    #[test]
    fn test_get_forge_path_debug() {
        let config = dummy_config(true);
        let input = PathBuf::from("src/main.c");
        let result = get_equivalent_forge_path(&input, &config, false).unwrap();
        let expected = std::env::current_dir().unwrap().join("forge/debug/main.o");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_forge_path_release() {
        let config = dummy_config(false);
        let input = PathBuf::from("src/utils.c");
        let result = get_equivalent_forge_path(&input, &config, false).unwrap();
        let expected = std::env::current_dir().unwrap().join("forge/release/utils.o");
        assert_eq!(result, expected);
    }

    #[test]
    fn test_get_forge_path_shared() {
        let config = dummy_config(true); 
        let input = PathBuf::from("src/libmath.c");
        let result = get_equivalent_forge_path(&input, &config, true).unwrap();
        let expected = std::env::current_dir().unwrap().join("forge/libs/obj/libmath.o");
        assert_eq!(result, expected);
    }
}
