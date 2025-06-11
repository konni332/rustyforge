use crate::utils::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::arguments::Command::{Rebuild, Run};
use crate::fs_utils::*;
use crate::config::{Config};
use crate::hashes::{cache_hash, get_cached_hash, hash};
use crate::ui::{print_melting, verbose_command, verbose_command_hard};
use rayon::prelude::*;
use colored::Colorize;


pub fn compile(config: &Config) -> Result<(), String>{
    // get all files to compile
    let files;
    
    // if the command is rebuild or a --clean run, compile all files
    // otherwise, only compile the files that have changed
    match &config.args.command { 
        Rebuild => {
            files = get_files_to_compile(config);
        },
        Run(options) => {
            if options.clean {
                files = config.forge.build.src.clone();
            }
            else {
                files = get_files_to_compile(config);
            }
        }
        _ => {
            files = get_files_to_compile(config);
        }
    };
    
    // compile all files (only gcc for now)
    if !files.is_empty() {
        print_melting();
    }
    files.par_iter().enumerate().try_for_each(|(index, file)| -> Result<(), String> {
        
        let source_path = find_file(&file).map_err(|_| format!("Could not find file: {}", file))?;
        let output_path = get_equivalent_forge_path(&source_path, &config)?;
        
        let mut cmd = Command::new("gcc");
        
        if config.args.debug {
            cmd.arg("-g").arg("-O0").arg("-Wall").arg("-Wextra").arg("-DDEBUG");
        }
        else if config.args.release {
            cmd.arg("-O3").arg("-Wall").arg("-Wextra").arg("-DRELEASE").arg("-DNDEBUG");
        }
        
        if let Some(cflags) = config.forge.build.cflags.clone() {
            for flag in cflags {
                if is_valid_cflag(&flag) {cmd.arg(flag);}
            }
        }
        
        cmd.arg("-c").arg(source_path);
        
        for include_dir in &config.forge.build.include_dirs {
            cmd.arg(format!("-I{}", include_dir));
        }
        if let Some(dependencies) = &config.forge.dependencies {
            for include_dir in &dependencies.include_dirs {
                cmd.arg(format!("-I{}", include_dir));
            }
        }
        
        cmd.arg("-o").arg(output_path);
        
        if config.args.verbose {
            verbose_command(&cmd);
        }
        else if config.args.verbose_hard { 
            verbose_command_hard(&cmd);
        }
        
        let output = cmd.output().expect("Failed to execute gcc");
        
        if !output.status.success() {
            eprintln!("Furnace not hot enough! Error compiling file: {}:\n{}", file, String::from_utf8_lossy(&output.stderr));
            return Err("Error compiling file".to_string());
        }
        else {
            println!("[{}]", &file.green())
        }
        Ok(())
    })?;
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
        let o_file = get_equivalent_forge_path(&file_path, &config)
            .expect(format!("Could not find equivalent forge path for file: {}", file).as_str());
        
        // check whether the .o file exists, if not, add it to the files to compile
        if o_file.exists() {
            // if yes, check the hashes of the .c and .h files
            // if the hashes are different, add the .c file to the files to compile
            let new_c_hash = hash(&file_path);
            let cached_c_hash = get_cached_hash(&file_path);
            match cached_c_hash { 
                Some(cached_hash)  => {
                    match new_c_hash {
                        Ok(new_hash) => {
                            if new_hash != cached_hash {
                                files.push(file.clone());
                                continue;
                            }
                        }
                        Err(_) => {
                            files.push(file.clone());
                            continue;
                        }
                    }
                }
                None => {
                    files.push(file.clone());
                    continue;
                }
            }
            
            for h_file in &h_files {
                let new_h_hash = hash(&h_file);
                let cached_h_hash = get_cached_hash(&h_file);

                let header_changed = match cached_h_hash {
                    Some(cached_hash) => {
                        match new_h_hash {
                            Ok(new_hash) => new_hash != cached_hash,
                            Err(_) => true,
                        }
                    },
                    None => true,
                };

                if header_changed {
                    files.push(file.clone());
                    cache_hash(&h_file, hash(&h_file).unwrap());
                    break; 
                }
            }
            
            for h_file in &h_files {
                cache_hash(&h_file, hash(&h_file).unwrap());
            }
            
            
        }
        else {
            files.push(file.clone());
        }
    }
    // cache the new hashes
    for file in &files {
        let file_path = find_file(file).unwrap();
        cache_hash(&file_path, hash(&file_path).unwrap());
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
    
    if config.args.verbose {
        verbose_command(&cmd);   
    }
    else if config.args.verbose_hard {
        verbose_command_hard(&cmd);  
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













