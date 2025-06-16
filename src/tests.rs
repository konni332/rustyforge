use std::path::{Path};
use crate::arguments::Command::{Rebuild};
use crate::arguments::{BuildOptions, ForgeArgs};
use crate::config::{Build, CompilerKind, Config, Forge, Project};

// usage is not recognized by rustfmt
#[allow(dead_code)]
pub fn dummy_config(debug: bool) -> Config {
    Config {
        compiler: CompilerKind::GCC,
        args: ForgeArgs {
            verbose: false,
            verbose_hard: false,
            command: Rebuild(BuildOptions {debug, compiler: None, release: !debug}),
        },
        forge: Forge {
            build: Build {
                compiler: Some("gcc".to_string()),
                output: "dummy".to_string(),
                cflags: None,
                ldflags: None,
                src: Vec::new(),
                include_dirs: Vec::new(),
            },
            project: Project {
                name: "dummy".to_string(),
                targets: vec!["bin".to_string()],
            },
            dependencies: None,
        }
    }
}

// usage is not recognized by rustfmt
#[allow(dead_code)]
fn clear_dir(dir: &Path) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                std::fs::remove_file(path)?;
            }
            else if path.is_dir() {
                clear_dir(&path)?;
            }
        }
    }
    Ok(())
}


#[cfg(test)]
mod integration_tests {
    use std::env;
    use std::path::PathBuf;
    use crate::compile::compile;
    use crate::fs_utils::init_hash_cache_json;
    use crate::linker::link;
    use super::*;
    
    #[test]
    fn test_valid_project_gcc(){
        let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let org_cwd = cwd.clone();
        let tests_path = cwd.join("tests").join("fixtures").join("valid_project");
        // make sure the build directory is empty
        let debug_path  = tests_path.join("forge").join("debug");
        // create the debug dir if it does not exist!
        if !debug_path.exists() {
            std::fs::create_dir_all(&debug_path).unwrap();
        }
        clear_dir(&debug_path).unwrap();
        
        env::set_current_dir(&tests_path).unwrap();
        
        let mut config = dummy_config(true);
        config.forge.build.src.push("lib.c".to_string());
        config.forge.build.src.push("main.c".to_string());
        config.forge.build.include_dirs.push("include".to_string());

        init_hash_cache_json(tests_path.join("forge").join(".forge")).unwrap();
        
        let compile_res = compile(&config);
        assert!(compile_res.is_ok());
        
        let link_res = link(&config);
        assert!(link_res.is_ok());

        let main_o_path = debug_path.join("main.o");
        let lib_o_path = debug_path.join("lib.o");
        #[cfg(target_os = "windows")]
        let exe_path = debug_path.join("dummy.exe");
        #[cfg(not(target_os = "windows"))]
        let exe_path = debug_path.join("dummy");
        assert!(main_o_path.exists());
        assert!(lib_o_path.exists());
        assert!(exe_path.exists());
        
        // delete the build contents
        clear_dir(&debug_path).unwrap();
        
        env::set_current_dir(org_cwd).unwrap();
    }
    
    #[test]
    fn test_broken_project_gcc(){
        let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let org_cwd = cwd.clone();
        let tests_path = cwd.join("tests").join("fixtures").join("broken_project");
        // make sure the build directory is empty
        let debug_path  = tests_path.join("forge").join("debug");
        clear_dir(&debug_path).unwrap();

        env::set_current_dir(&tests_path).unwrap();

        let mut config = dummy_config(true);
        config.forge.build.src.push("main.c".to_string());

        init_hash_cache_json(tests_path.join("forge").join(".forge")).unwrap();
        
        let compile_res = compile(&config);
        assert!(compile_res.is_err());
        env::set_current_dir(org_cwd).unwrap();
    }
    #[test]
    fn test_valid_project_clang(){
        let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let org_cwd = cwd.clone();
        let tests_path = cwd.join("tests").join("fixtures").join("valid_project");
        // make sure the build directory is empty
        let debug_path  = tests_path.join("forge").join("debug");
        // create the debug dir if it does not exist!
        if !debug_path.exists() {
            std::fs::create_dir_all(&debug_path).unwrap();
        }
        clear_dir(&debug_path).unwrap();

        env::set_current_dir(&tests_path).unwrap();

        let mut config = dummy_config(true);
        config.forge.build.src.push("lib.c".to_string());
        config.forge.build.src.push("main.c".to_string());
        config.forge.build.include_dirs.push("include".to_string());
        // set compiler to clang
        config.compiler = CompilerKind::Clang;
        
        init_hash_cache_json(tests_path.join("forge").join(".forge")).unwrap();
        
        let compile_res = compile(&config);
        assert!(compile_res.is_ok());

        let link_res = link(&config);
        assert!(link_res.is_ok());

        let main_o_path = debug_path.join("main.o");
        let lib_o_path = debug_path.join("lib.o");
        #[cfg(target_os = "windows")]
        let exe_path = debug_path.join("dummy.exe");
        #[cfg(not(target_os = "windows"))]
        let exe_path = debug_path.join("dummy");
        assert!(main_o_path.exists());
        assert!(lib_o_path.exists());
        assert!(exe_path.exists());

        // delete the build contents
        clear_dir(&debug_path).unwrap();

        env::set_current_dir(org_cwd).unwrap();
    }

    #[test]
    fn test_broken_project_clang(){
        let cwd = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let org_cwd = cwd.clone();
        let tests_path = cwd.join("tests").join("fixtures").join("broken_project");
        // make sure the build directory is empty
        let debug_path  = tests_path.join("forge").join("debug");
        clear_dir(&debug_path).unwrap();

        env::set_current_dir(&tests_path).unwrap();

        let mut config = dummy_config(true);
        config.forge.build.src.push("main.c".to_string());
        // set compiler to clang
        config.compiler = CompilerKind::Clang;

        init_hash_cache_json(tests_path.join("forge").join(".forge")).unwrap();
        
        let compile_res = compile(&config);
        assert!(compile_res.is_err());
        env::set_current_dir(org_cwd).unwrap();
    }
}