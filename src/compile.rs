use crate::utils::*;
use std::path::{Path, PathBuf};
use std::process::Command;
use crate::fs_utils::*;
use crate::config::{CompilerKind, Config};
use crate::hashes::{cache_hash, file_changed};
use crate::ui::{print_heating, verbose_command, verbose_command_hard};
use rayon::prelude::*;
use colored::Colorize;
use crate::arguments::Command::{Run, Rebuild};
use anyhow::{Result, bail, Context};

pub fn compile(config: &Config) -> Result<()>{
    if config.compiler == CompilerKind::MSVC{
        bail!("MSVC is not supported yet");   
    }
    
    // gcc / clang is handled in compile_unix_like()
    let targets = &config.forge.project.targets;
    if targets.contains(&"shared".to_string()) {
        compile_unix_like(config, true)?;
    }
    if targets.contains(&"static".to_string()) || targets.contains(&"bin".to_string()) {
        compile_unix_like(config, false)?;   
    }
    Ok(())
}

pub fn compile_unix_like(config: &Config, shared: bool) -> Result<()>{
    let to_compile = get_files_to_compile(config, shared)?;
    let files: Vec<String> = to_compile.iter().map(|(f, _)| f.clone()).collect();
    let h_files: Vec<PathBuf> = to_compile.iter()
        .flat_map(|(_, h)| h.clone()).collect();
    
    if shared {
        create_forge_sub_dir("libs/obj")?;
    }
    
    // compile all files (only gcc for now)
    if !files.is_empty() {
        print_heating();
    }
    
    files.par_iter().enumerate().try_for_each(|(_, file)| -> Result<()> {
        let source_path = find_file(&file)
            .map_err(|e| anyhow::Error::new(e))?;
        let output_path = get_equivalent_forge_path(&source_path, &config, shared)?;
        
        let mut cmd;
        if config.compiler == CompilerKind::GCC {
            cmd = Command::new("gcc");
        }
        else if config.compiler == CompilerKind::Clang {
            cmd = Command::new("clang");
        }
        else {
            bail!(format!("Compiler not supported: {}", config.compiler))  
        }
        
        // add target specific compiler flags
        add_build_flags(&config.args.command, &mut cmd);
        // if we are compiling a shared library, add the extra flags
        if shared {
            cmd.arg("-fPIC");
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
            eprintln!(
                "Furnace not hot enough! Error compiling file: {}:\n{}",
                file, String::from_utf8_lossy(&output.stderr)
            );
            bail!("Error compiling file: {}", file)
        }
        else {
            println!("[{}]", &file.green())
        }
        Ok(())
    })?;
    // cache .c hashes
    for c_file in &config.forge.build.src {
        let absolut_path = find_file(&c_file)?;
        cache_hash(&absolut_path, std_hash_cache_path()?)?;      
    }
    // cache .h hashes
    for h_file in &h_files {
        cache_hash(&h_file, std_hash_cache_path()
            .expect("Could not get std hash cache path"))?;      
    }
    Ok(())
}

pub fn get_files_to_compile(config: &Config, shared: bool)
    -> Result<Vec<(String, Vec<PathBuf>)>> 
{
    let mut to_compile= Vec::new();
    
    for c_file in &config.forge.build.src {
        // get all relevant file paths
        let c_file_path = find_file(&c_file)?;
        let o_file_path = get_equivalent_forge_path(&c_file_path, &config, shared).with_context(|| 
            format!("Could not get equivalent forge path for file: {}", c_file))?;
        let h_files = parse_h_dependencies(Path::new(c_file), &config)
            .with_context(|| format!("Could not parse dependencies for file: {}", c_file))?;
        let mut compile = false;
        // if command ist rebuild, compile all files
        match &config.args.command {
            Rebuild(_) => compile = true,
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
        if file_changed(&c_file_path, std_hash_cache_path()
                .expect("Could not get std hash cache path"))
            .with_context(|| format!("Could not check if file changed: {}", c_file))? 
        {
            compile = true;    
        }
        // check if any of the h files have changed
        for h_file in &h_files {
            if file_changed(&h_file, std_hash_cache_path()
                    .expect("Could not get std hash cache path"))
                .with_context(|| format!("Could not check if file changed: {}", h_file.display()))? 
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

fn gcc_clang_mm(relpath: &Path, config: &Config) -> Result<String> {
    let mut cmd = get_compiler_cmd(config)?;
    let mm_path = normalize_path(relpath);
    
    cmd.arg("-MM").arg(mm_path);
    for dir in &config.forge.build.include_dirs {
        cmd.arg(format!("-I{}", dir.clone()));
    }
    
    if config.args.verbose {
        verbose_command(&cmd);   
    }
    else if config.args.verbose_hard {
        verbose_command_hard(&cmd);  
    }
    
    let output = cmd.output()?;
    
    if output.status.success() {
        let stdout = String::from_utf8(output.stdout).with_context(|| 
            format!("Could not convert stdout to string: {}", relpath.display()))?;
        Ok(stdout)
    }
    else {
        let stderr = String::from_utf8(output.stderr)
            .with_context(|| format!("Could not convert stderr to string: {}", relpath.display()))?;
        bail!(format!("Error compiling file: {}", stderr)); 
    }
}

fn parse_h_dependencies(relpath: &Path, config: &Config) -> Result<Vec<PathBuf>> {
    let output = gcc_clang_mm(relpath, config)?;
    let parts: Vec<&str> = output.split(':').collect();
    if parts.len() != 2 {
        bail!(format!("Could not parse compiler output: {}", output));  
    }
    
    let deps_str = parts[1];
    
    let deps_str = deps_str.replace("\\\n", "");
    let deps = deps_str.split_whitespace();
    
    let cwd = std::env::current_dir()
        .expect("Could not get current working directory");
    
    let mut deps_paths = Vec::new();
    
    for dep in deps {
        if dep.ends_with(".h") {
            let path = Path::new(dep);
            let abs_path = if path.is_absolute() {
                path.to_path_buf()
            }
            else { 
                cwd.join(path).canonicalize()?
            };
            deps_paths.push(abs_path);
        }
    }
    
    Ok(deps_paths)
}


pub fn get_compiler_cmd(config: &Config) -> Result<Command> {
    if config.compiler == CompilerKind::GCC {
        Ok(Command::new("gcc"))
    }
    else if config.compiler == CompilerKind::Clang {
        Ok(Command::new("clang"))
    }
    else {
        bail!(format!("Compiler not supported: {}", config.compiler))  
    }
}










