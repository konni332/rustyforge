use std::path::Path;
use std::process::Command;
use crate::config::{CompilerKind, Config};
use crate::utils::{format_lib_name, is_valid_ldflag, format_shared_lib_name};
use crate::ui::{print_forging, verbose_command, verbose_command_hard};
use anyhow::{bail, Result};
use crate::compile::get_compiler_cmd;
use crate::fs_utils::{create_forge_sub_dir, normalize_path, find_o_files, find_o_files_dir};


#[allow(unused_imports)] // is imported for linux and macOS
use crate::fs_utils::{find_file, find_r_paths};

pub fn link(config: &Config) -> Result<()>{
    if config.compiler == CompilerKind::MSVC {
        bail!("MSVC is not supported yet");
    }
    // clang / gcc is handled by the link() function
    // check all targets
    for target in &config.forge.project.targets {
        match target.as_str() {
            "bin" => {
                if let Err(e) = link_executable(config){
                    eprintln!("Error: {}", e);
                }
            }
            "static" => {
                if let Err(e) = archive_static_library(config){
                    eprintln!("Error: {}", e);
                }
            }
            "shared" => {
                if let Err(e) = link_shared_library(config){
                    eprintln!("Error: {}", e);
                }
            }
            _ => {
                bail!("Unknown target: {} None of [bin, static, shared]", target);
            }
        }
    };
    Ok(())
}

pub fn link_shared_library(cfg: &Config) -> Result<()>{
    let lib_name = cfg.forge.build.output.clone();
    let mut formatted_name = lib_name.clone();
    format_shared_lib_name(&mut formatted_name);
    let out = format!("forge/libs/out/{}", formatted_name);
    
    create_forge_sub_dir("libs/out")?;
    
    let mut cmd= match get_compiler_cmd(&cfg) {
        Ok(cmd) => cmd,
        Err(e) => bail!("Failed to get compiler command: {}", e)
    };
    
    let o_path = Path::new("forge/libs/obj");
    let o_files = find_o_files(o_path);
    
    cmd.arg("-shared");
    #[cfg(target_os = "linux")]
    cmd.arg("-fPIC");
    cmd.arg("-o").arg(out);
    for o_file in &o_files {
        cmd.arg(o_file);
    }

    #[cfg(target_os = "windows")]
    {
        let windows_arg = format!("-Wl,--out-implib,forge/libs/out/lib{}.dll.a", lib_name);
        cmd.arg(windows_arg);
    }
    
    print_forging(&lib_name);
    if cfg.args.verbose {
        verbose_command(&cmd);
    }
    else if cfg.args.verbose_hard { 
        verbose_command_hard(&cmd);
    }
    
    let output = cmd.output().expect("Failed to run gcc");
    
    if !output.status.success() {
        bail!("Hammer to rusty, linker failed: {}", String::from_utf8_lossy(&output.stderr))
    }
    else { 
        println!("Forging successful!")
    }
    Ok(())
}

pub fn archive_static_library(cfg: &Config) -> Result<()>{
    // get a formatted name for the library, based on the output name, and the OS(Toolchain)
    let mut name = cfg.forge.build.output.clone();
    format_lib_name(&mut name);
    name = format!("forge/libs/out/{}", name);
    
    create_forge_sub_dir("libs/out")?;
    
    let mut cmd = Command::new("ar");
    cmd.arg("rcs").arg(&name);
    
    let o_path = find_o_files_dir(&cfg); 
    let o_files = find_o_files(&o_path);
    for o_file in &o_files {
        // add the normalized path
        cmd.arg(normalize_path(o_file));
    }

    print_forging(&name);
    if cfg.args.verbose {
        verbose_command(&cmd);
    }
    else if cfg.args.verbose_hard { 
        verbose_command_hard(&cmd);
    }
    
    let output = cmd.output().expect("Failed to run ar");
    
    if !output.status.success() {
        bail!("Hammer to rusty, linker failed: {}", String::from_utf8_lossy(&output.stderr))
    }
    else { 
        println!("Forging successful!")
    }
    Ok(())
}

pub fn link_executable(config: &Config) -> Result<()> {
    let target_executable = if cfg!(target_os = "windows") {
        format!("{}.exe", config.forge.build.output)
    }
    else { 
        config.forge.build.output.clone()
    };
    
    
    let o_path = find_o_files_dir(&config);
    let o_files = find_o_files(&o_path);
    
    print_forging(&target_executable);
    let cwd = std::env::current_dir().expect("Failed to get current directory");
    
    let target_path= cwd.join(find_o_files_dir(&config).join(target_executable.clone()));
    
    let mut cmd= match get_compiler_cmd(&config) {
        Ok(cmd) => cmd,
        Err(e) => bail!("Failed to get compiler command: {}", e)
    };
    
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
        
        for lib in &dependencies.posix_libraries {
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
        bail!("Hammer to rusty, linker failed: {}", String::from_utf8_lossy(&output.stderr))
    }
    else { 
        println!("Forging successful!")
    }
    Ok(())
}

