use std::path::PathBuf;
use std::process::Command;
use crate::config::Forge;
use crate::fs_utils::find_file;

pub fn find_o_files() -> Vec<PathBuf>{
    let mut cwd = std::env::current_dir().expect("Failed to get current directory");
    cwd.push("forge");
    
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

pub fn link(config: &Forge){
    let target_executable = if cfg!(target_os = "windows") {
        format!("{}.exe", config.build.output)
    }
    else { 
        config.build.output.clone()
    };
    
    let o_files = find_o_files();
    // TODO: Also link libs etc.
    
    println!("Forging...\n{}", target_executable);
    let cwd = std::env::current_dir().expect("Failed to get current directory");
    let target_path = cwd.join("forge").join(target_executable);
    
    let mut cmd = Command::new("gcc");
    for o_file in o_files {
        cmd.arg(o_file);
    }
    cmd.arg("-o").arg(target_path);
    let output = cmd.output().expect("Failed to run gcc");
    if !output.status.success() {
        eprintln!("Hammer to rusty, linker failed: {}", String::from_utf8_lossy(&output.stderr))
    }
    else { 
        println!("Forging successful!")
    }
}