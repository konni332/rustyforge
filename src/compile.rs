use crate::utils::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::fs_utils::*;
use crate::config::{Config};
use crate::hashes::{cache_hash, file_changed};
use crate::ui::{print_melting, verbose_command, verbose_command_hard};
use rayon::prelude::*;
use colored::Colorize;
use crate::arguments::Command::{Run, Rebuild};

pub fn compile(config: &Config) -> Result<(), String>{
    let to_compile = get_files_to_compile(config)?;
    let files: Vec<String> = to_compile.iter().map(|(f, _)| f.clone()).collect();
    let h_files: Vec<PathBuf> = to_compile.iter().flat_map(|(_, h)| h.clone()).collect();
    
    // compile all files (only gcc for now)
    if !files.is_empty() {
        print_melting();
    }
    files.par_iter().enumerate().try_for_each(|(index, file)| -> Result<(), String> {
        
        let source_path = find_file(&file).map_err(|_| format!("Could not find file: {}", file))?;
        let output_path = get_equivalent_forge_path(&source_path, &config)?;
        
        let mut cmd = Command::new("gcc");
        
        if config.args.debug {
            add_debug_cflags(&mut cmd);
        }
        else if config.args.release {
            add_release_cflags(&mut cmd);
        }
        
        if let Some(cflags) = config.forge.build.cflags.clone() {
            for flag in cflags {
                if is_valid_cflag(&flag) {cmd.arg(flag);}
            }
        }
        
        cmd.arg("-c").arg(source_path.clone());
        
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
    // cache .c hashes
    for c_file in &config.forge.build.src {
        let absolut_path = find_file(&c_file)
            .map_err(|_| format!("Could not find file: {}", c_file))?;
        cache_hash(&absolut_path)?;      
    }
    // cache .h hashes
    for h_file in &h_files {
        cache_hash(&h_file)?;      
    }
    Ok(())
}

pub fn get_files_to_compile(config: &Config) -> Result<Vec<(String, Vec<PathBuf>)>, String> {
    let mut to_compile= Vec::new();
    
    for c_file in &config.forge.build.src {
        // get all relevant file paths
        let c_file_path = find_file(&c_file)
            .map_err(|_| format!("Could not find file: {}", c_file))?;
        let o_file_path = get_equivalent_forge_path(&c_file_path, &config)?;
        let h_files = parse_h_dependencies(Path::new(c_file), &config)
            .map_err(|_| format!("Could not parse header dependencies for file: {}", c_file))?;
        let mut compile = false;
        // if command ist rebuild, compile all files
        match &config.args.command {
            Rebuild => compile = true,
            Run(options) => {
                if options.clean {
                    compile = true;
                }
            },
            _ => {}
        }
        
        // if the o file doesn't exist, compile
        if !o_file_path.exists() {
            compile = true;
        }
        // if the c file has changed, compile
        if file_changed(&c_file_path)
            .map_err(|_| format!("Could not check if file changed: {}", c_file))? 
        {
            compile = true;    
        }
        // check if any of the h files have changed
        for h_file in &h_files {
            if file_changed(&h_file)
                .map_err(|_| format!("Could not check if file changed: {}", h_file.display()))? 
            {
                compile = true;
                break;
            }
        }
        
        // if compile is true, add the c file to the list of files to compile
        if compile {
            to_compile.push((c_file.clone(), h_files));
        }
    }
    Ok(to_compile)
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
                cwd.join(path).canonicalize()
                    .map_err(|e| format!("Could not canonicalize path: {}", e).to_string())?
            };
            deps_paths.push(abs_path);
        }
    }
    Ok(deps_paths)
}













