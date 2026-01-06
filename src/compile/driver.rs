use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    path::{Path, PathBuf},
    process::Command,
    sync::atomic::{AtomicBool, Ordering},
};

use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use indicatif::{ProgressBar, ProgressStyle};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
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
    ui::{self, discovered_dir_msg, discovered_file_msg},
};

pub struct CompilerDriver<C: Compiler + Sync> {
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

impl<C: Compiler + Sync> CompilerDriver<C> {
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
        let ignore_pattern_strings = &project_config.build.ignore_patterns;
        let user_flags = match profile {
            Profile::Debug => project_config
                .project
                .profile_debug
                .as_ref()
                .and_then(|p| p.flags.clone())
                .unwrap_or(vec![]),
            Profile::Release => project_config
                .project
                .profile_release
                .as_ref()
                .and_then(|p| p.flags.clone())
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
        for entry in jwalk::WalkDir::new(".")
            .skip_hidden(true)
            .process_read_dir(|depth, _path, _state, entries| {
                if let Some(depth) = depth
                    && depth >= 1
                {
                    for entry in entries.iter_mut().flatten() {
                        if entry.path().join("RustyForge.toml").is_file() {
                            entry.read_children_path = None; // Unterbaum ignorieren
                        }
                    }
                }
            })
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().map(|ext| ext == "c").unwrap_or(false)
                && !self.should_be_ignored(&path)
            {
                if get_verbosity!() > 0 {
                    discovered_file_msg(&path);
                }
                files.push(path.to_path_buf());
            }
        }
        Ok(files)
    }
    fn discover_include_dirs(&self) -> Result<Vec<PathBuf>> {
        let mut dirs = HashMap::<PathBuf, ()>::new();

        for entry in jwalk::WalkDir::new(".")
            .skip_hidden(true)
            .process_read_dir(|depth, _path, _state, entries| {
                if let Some(depth) = depth
                    && depth >= 1
                {
                    for entry in entries.iter_mut().flatten() {
                        if entry.path().join("RustyForge.toml").is_file() {
                            entry.read_children_path = None; // Unterbaum ignorieren
                        }
                    }
                }
            })
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            if path.extension().map(|e| e == "h").unwrap_or(false) {
                let mut current = path.parent();

                while let Some(parent) = current {
                    if dirs.insert(parent.to_path_buf(), ()).is_some() {
                        break;
                    }

                    if get_verbosity!() > 0 {
                        discovered_dir_msg(parent);
                    }

                    current = parent.parent();
                }
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
            if !dep.exists() {
                continue;
            }
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
        let cmds: Result<Vec<(PathBuf, u64, std::process::Command)>> = files
            .into_par_iter()
            .map(|file| {
                let unit = CompileUnit {
                    source: &file,
                    includes: &includes,
                    defines: &defines,
                };
                let mut deps = self.compiler.get_dependencies(&unit)?;

                deps.retain(|pb| !self.should_be_ignored(pb));
                deps.sort();
                let cmd = self.compiler.compile_cmd(&unit, &self.opts)?;
                let hash = self.compute_command_hash(&cmd, &file, &deps)?;
                if self.cache.map.get(&hash).is_some_and(|p| *p == file) {
                    return Ok(None);
                }

                Ok(Some((file.to_path_buf(), hash, Command::from(&cmd))))
            })
            .filter_map(|res| match res {
                Ok(Some(v)) => Some(Ok(v)),
                Ok(None) => None,
                Err(e) => Some(Err(e)),
            })
            .collect();
        let cmds = cmds?;

        Ok(cmds)
    }

    pub fn compile_incremental(&mut self) -> Result<()> {
        let cmds = self.resolve_incremental()?;
        let failed = AtomicBool::new(false);

        let pb = ProgressBar::new(cmds.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40}] {pos}/{len} ({eta}) {msg}",
                )
                .context("Failed to create progressbar")?
                .progress_chars("#>-"),
        );

        let updates: Vec<(u64, PathBuf)> = cmds
            .into_par_iter()
            .filter_map(|(path, hash, mut cmd)| {
                let output = match cmd.output() {
                    Ok(o) => o,
                    Err(e) => {
                        let msg = ui::error_compile_msg(&path, &cmd, format!("{}", e).as_bytes());
                        pb.println(msg);
                        failed.store(true, Ordering::Relaxed);
                        pb.inc(1);
                        return None;
                    }
                };

                if output.status.success() {
                    let msg = ui::successfull_compile_msg(&path, &cmd);
                    pb.println(msg);
                    pb.inc(1);
                    Some((hash, path))
                } else {
                    let msg = ui::error_compile_msg(&path, &cmd, &output.stderr);
                    pb.println(msg);
                    failed.store(true, Ordering::Relaxed);
                    pb.inc(1);
                    None
                }
            })
            .collect();

        pb.finish_and_clear();
        println!("{}", ui::finished_compilation_msg(pb.elapsed()));

        for (hash, path) in updates {
            self.cache.map.insert(hash, path);
        }

        if failed.load(Ordering::Relaxed) {
            anyhow::bail!("compilation failed");
        }

        Ok(())
    }
}

impl<C: Compiler + Sync> Drop for CompilerDriver<C> {
    fn drop(&mut self) {
        self.cache.save_cache().ok();
    }
}
