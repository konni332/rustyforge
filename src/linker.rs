use std::path::PathBuf;
use std::process::Command;
use crate::config::{Config};
#[allow(unused_imports)] // is imported for linux and macOS
use crate::fs_utils::{find_file, find_r_paths};
use crate::utils::{is_valid_ldflag};
use crate::ui::{print_forging, verbose_command, verbose_command_hard};

pub fn find_o_files(config: &Config) -> Vec<PathBuf>{
    let mut cwd = std::env::current_dir().expect("Failed to get current directory");
    cwd.push("forge");
    
    if config.args.debug {
        cwd.push("debug");
    }
    else {
        cwd.push("release");
    }
    
    let mut o_files = Vec::new();
    
    for entry in std::fs::read_dir(cwd).expect("Failed to read forge/ directory") {
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

pub fn link(config: &Config){
    let target_executable = if cfg!(target_os = "windows") {
        format!("{}.exe", config.forge.build.output)
    }
    else { 
        config.forge.build.output.clone()
    };
    
    let o_files = find_o_files(&config);
    // TODO: Also link libs etc.
    
    print_forging(&target_executable);
    let cwd = std::env::current_dir().expect("Failed to get current directory");
    
    let target_path;
    if config.args.debug {
        target_path = cwd.join("forge").join("debug").join(target_executable);
    }
    else {
        target_path = cwd.join("forge").join("release").join(target_executable);
    }
    
    let mut cmd = Command::new("gcc");
    // add all object files
    for o_file in o_files {
        cmd.arg(o_file);
    }
    
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    let r_paths = find_r_paths(&config);
    // add all library paths
    if let Some(dependencies) = &config.forge.dependencies {
        
        for lib_path in &dependencies.library_paths {
            cmd.arg(format!("-L{}", lib_path));
        }
        
        // add all rpaths (only linux and macOS)
        #[cfg(any(target_os = "linux", target_os = "macos"))]
        for path in &r_paths{
            cmd.arg(format!("-Wl,-rpath={}", path.to_str().unwrap()).as_str());
        }
        
        // add all libraries
        for lib in &dependencies.libraries {
            cmd.arg(format!("-l{}", lib));
        }
    }
    cmd.arg("-o").arg(target_path);
    
    // add user ldflags
    if let Some(ldflags) = &config.forge.build.ldflags.clone() {
        for flag in ldflags {
            if is_valid_ldflag(flag) { cmd.arg(flag); }
        }
    }
    
    if config.args.verbose {
        verbose_command(&cmd);
    }
    else if config.args.verbose_hard { 
        verbose_command_hard(&cmd);
    }
    
    let output = cmd.output().expect("Failed to run gcc");
    
    if !output.status.success() {
        eprintln!("Hammer to rusty, linker failed: {}", String::from_utf8_lossy(&output.stderr))
    }
    else { 
        println!("Forging successful!")
    }
}