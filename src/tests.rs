use crate::arguments::Command::Clean;
use crate::arguments::ForgeArgs;
use crate::config::{Build, Config, Forge, Project};

pub fn dummy_config(debug: bool) -> Config {
    Config {
        args: ForgeArgs {
            debug,
            release: !debug,
            verbose: false,
            verbose_hard: false,
            command: Clean,
        },
        forge: Forge {
            build: Build {
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