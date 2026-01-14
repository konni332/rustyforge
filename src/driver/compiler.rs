use globset::{Glob, GlobSet, GlobSetBuilder};
use rayon::prelude::*;
use std::ffi::OsStr;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use crate::CoreError;
use crate::driver::cache::{BuildCache, CacheFile};
use crate::driver::runtime::{Profile, RuntimeToolchain};
use crate::{
    CoreResult,
    config::tool_config::{CCompilerKind, CppCompilerKind},
    driver::{
        GlobalContext,
        cannonical_command::CannonicalCommand,
        runtime::Target,
        toolchain::{CCompiler, Clang, CppCompiler, Gcc, Intel, Msvc},
    },
    internal_error,
};

pub struct CompileContext<'ctx> {
    c_compiler: CCompilerKind,
    cpp_compiler: CppCompilerKind,
    includes: &'ctx [PathBuf],
    c_files: &'ctx [PathBuf],
    cpp_files: &'ctx [PathBuf],
    target: &'ctx Target<'ctx>,
    profile: &'ctx Profile,
    build_cache: &'ctx mut CacheFile<BuildCache>,
}

impl<'ctx> CompileContext<'ctx> {
    pub fn new(
        includes: &'ctx [PathBuf],
        c_files: &'ctx [PathBuf],
        cpp_files: &'ctx [PathBuf],
        target: &'ctx Target<'ctx>,
        profile: &'ctx Profile,
        toolchain: &'ctx RuntimeToolchain,
        build_cache: &'ctx mut CacheFile<BuildCache>,
    ) -> Self {
        let c_compiler = toolchain.c_compiler;
        let cpp_compiler = toolchain.cpp_compiler;
        Self {
            c_compiler,
            cpp_compiler,
            includes,
            c_files,
            cpp_files,
            target,
            profile,
            build_cache,
        }
    }
    pub fn build(&self, obj_dir: &Path) -> CoreResult<CompileResult> {
        let mut c_commands = match self.c_compiler {
            CCompilerKind::Clang => self.build_c_commands::<Clang>(obj_dir),
            CCompilerKind::Gcc => self.build_c_commands::<Gcc>(obj_dir),
            CCompilerKind::Msvc => self.build_c_commands::<Msvc>(obj_dir),
            CCompilerKind::Icc => self.build_c_commands::<Intel>(obj_dir),
        }?;
        let cpp_commands = match self.cpp_compiler {
            CppCompilerKind::Clangpp => self.build_cpp_commands::<Clang>(obj_dir),
            CppCompilerKind::Gpp => self.build_cpp_commands::<Gcc>(obj_dir),
            CppCompilerKind::Msvc => self.build_cpp_commands::<Msvc>(obj_dir),
            CppCompilerKind::Icpc => self.build_cpp_commands::<Intel>(obj_dir),
        }?;
        c_commands.extend(cpp_commands);
        Ok(CompileResult {
            cmds: c_commands,
            has_cpp: !self.cpp_files.is_empty(),
        })
    }
    fn build_c_commands<C: CCompiler + Send + Sync>(
        &self,
        obj_dir: &Path,
    ) -> CoreResult<Vec<(CannonicalCommand, u64)>> {
        let mut results = vec![];
        let comp = C::new();
        for src in self.c_files {
            let dependencies =
                comp.get_dependencies(src, self.profile, self.target, self.includes)?;

            let output = output(src, obj_dir);
            let cmd =
                comp.compile_unit_cmd(src, &output, self.profile, self.target, self.includes)?;
            let hash = get_tu_hash(src, &cmd, &dependencies, self.build_cache.seed())?;
            results.push((cmd, hash));
        }
        Ok(results)
    }
    fn build_cpp_commands<C: CppCompiler + Send + Sync>(
        &self,
        obj_dir: &Path,
    ) -> CoreResult<Vec<(CannonicalCommand, u64)>> {
        let mut results = vec![];
        let comp = C::new();
        for src in self.cpp_files {
            let dependencies =
                comp.get_dependencies(src, self.profile, self.target, self.includes)?;

            let output = output(src, obj_dir);
            let cmd =
                comp.compile_unit_cmd(src, &output, self.profile, self.target, self.includes)?;
            let hash = get_tu_hash(src, &cmd, &dependencies, self.build_cache.seed())?;
            results.push((cmd, hash));
        }
        Ok(results)
    }
}

pub struct CompileResult {
    pub cmds: Vec<(CannonicalCommand, u64)>,
    pub has_cpp: bool,
}

fn output(src: &Path, obj_dir: &Path) -> PathBuf {
    let stem = match src.file_stem() {
        Some(s) => s,
        None => {
            internal_error!(
                "Failed to get file stem form src file, this can only occur if discovery has been corrupted"
            );
        }
    };

    #[cfg(not(windows))]
    return obj_dir.join(format!("{}.o", stem.display()));
    #[cfg(windows)]
    return obj_dir.join(format!("{}.obj", stem.display()));
}

fn get_tu_hash(
    src: &Path,
    cmd: &CannonicalCommand,
    dependencies: &[PathBuf],
    seed: u64,
) -> CoreResult<u64> {
    let mut hasher = twox_hash::XxHash64::with_seed(seed);
    let src_content = std::fs::read(src)?;

    src_content.hash(&mut hasher);

    cmd.hash(&mut hasher);

    let mut dependencies = dependencies.to_vec();
    dependencies.sort();
    for dep in dependencies {
        let dep_content = std::fs::read(dep)?;
        dep_content.hash(&mut hasher);
    }

    Ok(hasher.finish())
}
