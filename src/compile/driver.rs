use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};
use twox_hash::XxHash64;
use verbosio::get_verbosity;

use crate::{
    cli::ForgeArgs,
    compile::{
        Compiler,
        types::{CannonicalCommand, CompileOptions, CompileUnit, Profile},
    },
    config::project::ProjectConfig,
    fs::{build_cache_path, debug_dir, object_dir, release_dir},
    ui::{self, output_discovered_dir, output_discovered_file},
};

pub struct CompilerDriver<C: Compiler> {
    pub compiler: C,
    pub ignore_set: GlobSet,
    pub opts: CompileOptions,
    cache: BuildCache,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct BuildCache {
    map: HashMap<u64, PathBuf>,
}

impl BuildCache {
    pub fn save_cache(&self) -> Result<()> {
        let bytes = postcard::to_allocvec(&self)?;
        let path = build_cache_path(std::env::current_dir()?);
        std::fs::write(path, bytes).context("Failed to write cache to disk")?;
        Ok(())
    }
    pub fn load_cache() -> Result<Self> {
        let path = build_cache_path(std::env::current_dir()?);
        if !path.exists() {
            return Ok(Self::default());
        }

        let bytes = std::fs::read(path).context("Failed to read cache from disk")?;
        let cache: Self = postcard::from_bytes(&bytes).context("Failed to deserialize cache")?;
        Ok(cache)
    }
}

impl<C: Compiler> CompilerDriver<C> {
    pub fn default_driver<P: AsRef<Path>>(
        root: P,
        args: &ForgeArgs,
        project_config: &ProjectConfig,
    ) -> Result<Self> {
        let profile = args.profile().unwrap_or(Profile::Debug);
        let profile_dir = match profile {
            Profile::Debug => debug_dir(root),
            Profile::Release => release_dir(root),
        };
        let object_dir = object_dir(profile_dir);
        std::fs::create_dir_all(&object_dir)?;
        let ignore_pattern_strings = &project_config.compilation.ignore_patterns;
        let user_flags = match profile {
            Profile::Debug => project_config
                .project
                .profile_debug
                .as_ref()
                .map(|p| p.flags.clone())
                .unwrap_or(vec![]),
            Profile::Release => project_config
                .project
                .profile_release
                .as_ref()
                .map(|p| p.flags.clone())
                .unwrap_or(vec![]),
        };
        let defines = match profile {
            Profile::Debug => project_config
                .project
                .profile_debug
                .as_ref()
                .map(|p| p.defines.clone())
                .unwrap_or(vec![]),
            Profile::Release => project_config
                .project
                .profile_release
                .as_ref()
                .map(|p| p.defines.clone())
                .unwrap_or(vec![]),
        };

        let target = args.target();
        Self::new(
            C::new(),
            object_dir,
            ignore_pattern_strings,
            profile,
            user_flags,
            defines,
            target.cloned(),
        )
    }
    pub fn new(
        compiler: C,
        object_dir: PathBuf,
        ignore_pattern_strings: &[String],
        profile: Profile,
        user_flags: Vec<String>,
        defines: Vec<String>,
        target: Option<String>,
    ) -> Result<Self> {
        let mut builder = GlobSetBuilder::new();
        for pat in ignore_pattern_strings {
            builder.add(Glob::new(pat)?);
        }
        let ignore_set = builder.build()?;
        Ok(Self {
            compiler,
            ignore_set,
            opts: CompileOptions {
                profile,
                object_dir,
                user_flags,
                target,
                defines,
            },
            cache: BuildCache::load_cache()?,
        })
    }
    fn discover_files(&self) -> Result<Vec<PathBuf>> {
        let mut files = vec![];
        for entry in walkdir::WalkDir::new(".")
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().map(|ext| ext == "c").unwrap_or(false)
                && !self.should_be_ignored(path)
            {
                if get_verbosity!() > 0 {
                    output_discovered_file(path);
                }
                files.push(path.to_path_buf());
            }
        }
        Ok(files)
    }
    fn discover_include_dirs(&self) -> Result<Vec<PathBuf>> {
        let mut dirs = HashMap::<PathBuf, ()>::new();

        for entry in walkdir::WalkDir::new(".")
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().map(|e| e == "h").unwrap_or(false)
                && let Some(parent) = path.parent()
                && dirs.insert(parent.to_path_buf(), ()).is_some()
                && get_verbosity!() > 0
            {
                output_discovered_dir(parent);
            }
        }

        Ok(dirs.into_keys().collect())
    }
    fn should_be_ignored(&self, path: &Path) -> bool {
        self.ignore_set.is_match(path)
    }
    fn compute_command_hash(
        &self,
        cmd: &CannonicalCommand,
        source: &Path,
        deps: &[PathBuf],
    ) -> Result<u64> {
        let mut hasher = XxHash64::default();

        cmd.hash(&mut hasher);

        let content = std::fs::read(source).context("Failed to read file contents")?;
        content.hash(&mut hasher);

        for dep in deps {
            let dep_content = std::fs::read(dep).context(format!(
                "Failed to read dependency content from: {}",
                dep.display()
            ))?;
            dep_content.hash(&mut hasher);
        }

        Ok(hasher.finish())
    }
    pub fn resolve_incremental(&mut self) -> Result<Vec<(PathBuf, u64, std::process::Command)>> {
        let mut files = self.discover_files()?;
        files.sort();
        let mut includes = self.discover_include_dirs()?;
        includes.sort();
        let mut defines = self.opts.defines.clone();
        defines.sort();
        let mut cmds = Vec::new();

        for file in files {
            let mut deps = self.compiler.get_dependencies(&file)?;
            deps.sort();

            let unit = CompileUnit {
                source: &file,
                includes: &includes,
                defines: &defines,
            };

            let cmd = self.compiler.compile_cmd(&unit, &self.opts)?;
            let hash = self.compute_command_hash(&cmd, &file, &deps)?;
            if self.cache.map.get(&hash).is_some_and(|p| *p == file) {
                continue;
            }

            cmds.push((file.to_path_buf(), hash, Command::from(&cmd)));
        }

        Ok(cmds)
    }

    pub fn compile_incremental(&mut self) -> Result<()> {
        let mut failed = false;
        let cmds = self.resolve_incremental()?;
        for (path, hash, mut cmd) in cmds {
            let output = cmd.output()?;
            if output.status.success() {
                ui::output_successfull_compile(&path, &cmd);
                self.cache.map.insert(hash, path.clone());
            } else {
                ui::output_error_compile(&path, &cmd, &output.stderr);
                failed = true;
            }
        }
        if failed {
            anyhow::bail!("compilation failed");
        }
        Ok(())
    }
}

impl<C: Compiler> Drop for CompilerDriver<C> {
    fn drop(&mut self) {
        self.cache.save_cache().ok();
    }
}
