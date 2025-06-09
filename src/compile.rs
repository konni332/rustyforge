use crate::utils::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::fs_utils::*;
use crate::config::{Config};
use crate::fs_utils::FileError::FileNotFound;


pub fn compile(config: &Config) -> Result<(), String>{
    // get all files to compile
    let files;
    
    if !config.args.rebuild_all {
        files = get_files_to_compile(config);
    }
    else {
        files = config.forge.build.src.clone();
    }
    
    // compile all files (only gcc for now)
    println!("Melting...");
    let mut files_compiled = 0;
    for file in files {
        let source_path = find_file(&file).expect(format!("Could not find file: {}", file).as_str());
        let output_path = get_equivalent_forge_path(&source_path)?;
        
        let mut cmd = Command::new("gcc");
        
        // compiler flags based on the build type
        if config.args.debug {
            cmd.arg("-g").arg("-O0").arg("-Wall").arg("-Wextra").arg("-DDEBUG");
        }
        else if config.args.release {
            cmd.arg("-O3").arg("-Wall").arg("-Wextra").arg("-DRELEASE").arg("-DNDEBUG");
        }
        
        // check for user flags
        if let Some(cflags) = config.forge.build.cflags.clone() {
            for flag in cflags {
                if is_valid_cflag(&flag) {cmd.arg(flag);}
            }
        }
        
        // compiler args
        cmd.arg("-c").arg(source_path);
        
   
        for include_dir in &config.forge.build.include_dirs {
            cmd.arg(format!("-I{}", include_dir));
        }
        if let Some(dependencies) = &config.forge.dependencies {
            for include_dir in &dependencies.include_dirs {
                cmd.arg(format!("-I{}", include_dir));
            }
        }
        // output flag
        cmd.arg("-o").arg(output_path);
        
        
        
        let output = cmd.output().expect("Failed to execute gcc");
        
        if !output.status.success() {
            eprintln!("Furnace not hot enough! Error compiling file: {}:\n{}", file, String::from_utf8_lossy(&output.stderr));
            return Err("Error compiling file".to_string());
        }
        else {
            files_compiled += 1;
            println!("[{}]: {}", files_compiled, file);
        }
    
    }
    Ok(())
}


// TODO: dry run function

fn get_files_to_compile(config: &Config) -> Vec<String> {
    let mut files = Vec::new();
    
    // TODO
    
    for file in &config.forge.build.src {
        // get the path to the .c file
        let file_path = find_file(&file)
            .expect(format!("Could not find file: {}", file).as_str());
        
        // get all .h files that are included in the .c file
        let rel_path: &Path = Path::new(file);
        let h_files = parse_h_dependencies(rel_path, &config)
            .expect("Could not resolve header dependencies");
        
        // generate the equivalent forge path for the .c file
        let o_file = get_equivalent_forge_path(&file_path)
            .expect(format!("Could not find equivalent forge path for file: {}", file).as_str());
        
        // check if the .o file is newer than the .c file or any of the .h files if it exists
        match find_file(o_file.to_str().expect("Could not convert path to string")) {
            Ok(o_path) => {
                
                // TODO: replace with a hash of the file contents at some point
                let c_file_time = get_timestamp(file_path);
                let o_file_time = get_timestamp(o_path);
                if c_file_time > o_file_time {
                    files.push(file.clone());
                    continue;
                }
                
                for h_file in &h_files {
                    let h_file_time = get_timestamp(h_file.clone());
                    if h_file_time > o_file_time {
                        files.push(file.clone());
                        break;
                    }
                }
            }
            Err(e) => {
                match e { 
                    // file does not exist? No problem, just add it to the list of files to compile
                    FileNotFound(_) => {
                        files.push(file.clone());
                    },
                    _ => {
                        panic!("Could not find file: {}", file);
                    }
                }
            }
        }
        
    }
    
    files
}

fn gcc_mm(relpath: &Path, config: &Config) -> Result<String, String> {
    let mut cmd = Command::new("gcc");
    
    let gcc_path = normalize_path(relpath);
    
    cmd.arg("-MM").arg(gcc_path);
    for dir in &config.forge.build.include_dirs {
        cmd.arg(format!("-I{}", dir.clone()));
    }
    
    let output = cmd.output().map_err(|e| format!("Command Error: {}", e).to_string())?;
    
    if output.status.success() {
        let stdout = String::from_utf8(output.stdout)
            .map_err(|e| format!("Could not convert stdout to string: {}", e).to_string())?;
        Ok(stdout)
    }
    else {
        let stderr = String::from_utf8(output.stderr)
            .map_err(|e| format!("Could not convert stderr to string: {}", e).to_string())?;
        Err(format!("gcc Error: {}", stderr).to_string())
    }
}

fn parse_h_dependencies(relpath: &Path, config: &Config) -> Result<Vec<PathBuf>, String> {
    let output = gcc_mm(relpath, config)?;
    let parts: Vec<&str> = output.split(':').collect();
    if parts.len() != 2 {
        return Err(format!("Could not parse gcc output: {}", output).to_string());
    }
    
    let deps_str = parts[1];
    
    let deps_str = deps_str.replace("\\\n", "");
    let deps = deps_str.split_whitespace();
    
    let cwd = std::env::current_dir().expect("Could not get current working directory");
    
    let mut deps_paths = Vec::new();
    
    for dep in deps {
        if dep.ends_with(".h") {
            let path = Path::new(dep);
            let abs_path = if path.is_absolute() {
                path.to_path_buf()
            }
            else { 
                cwd.join(path).canonicalize().map_err(|e| format!("Could not canonicalize path: {}", e).to_string())?
            };
            deps_paths.push(abs_path);
        }
    }
    Ok(deps_paths)
}













