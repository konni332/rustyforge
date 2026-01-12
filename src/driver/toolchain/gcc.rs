use std::{
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    CoreError, CoreResult, TargetKind,
    driver::{
        cannonical_command::CannonicalCommandBuilder,
        toolchain::{
            PROFILE_DEFINE_TEMPLATE,
            clang::build_command_compile,
            traits::{Archiver, CCompiler, CppCompiler, Linker},
        },
    },
    internal_error,
    utils::display_command,
    warn,
};

pub struct Gcc;

fn gcc_id() -> &'static str {
    "gcc"
}

impl CCompiler for Gcc {
    fn new() -> Self
    where
        Self: Sized,
    {
        Gcc
    }
    fn get_dependencies(
        &self,
        src: &Path,
        profile: &crate::driver::runtime::Profile,
        target: &crate::driver::runtime::Target,
        includes: &[PathBuf],
    ) -> CoreResult<Vec<PathBuf>> {
        let mut cmd = CannonicalCommandBuilder::new("gcc");
        cmd.arg("-MM").arg(src);
        let cmd = build_command_compile(src, cmd, profile, target, includes)?;
        let output = Command::from(&cmd).output()?;

        if !output.status.success() {
            return Err(Box::new(CoreError::ResolveDependency {
                err: String::from_utf8_lossy(&output.stderr).to_string(),
                cmd: display_command(&std::process::Command::from(&cmd)),
            }));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        let mut deps = Vec::new();
        for line in stdout.lines() {
            let line = line.trim_end_matches('\\').trim();
            if let Some(pos) = line.find(':') {
                let files = &line[pos + 1..];
                deps.extend(
                    files
                        .split_whitespace()
                        .map(|s| {
                            let path = PathBuf::from(s);
                            match path.canonicalize() {
                                Ok(p) => Ok(p),
                                Err(e) => {
                                    warn!(&format!(
                                        "Failed to cannonicalize dependency path: {}",
                                        path.display()
                                    ));
                                    Err(e)
                                }
                            }
                        })
                        .filter_map(|res| res.ok()),
                );
            }
        }

        Ok(deps)
    }
    fn compile_unit_cmd(
        &self,
        path: &std::path::Path,
        output: &std::path::Path,
        profile: &crate::driver::runtime::Profile,
        target: &crate::driver::runtime::Target,
        includes: &[PathBuf],
    ) -> crate::CoreResult<crate::driver::cannonical_command::CannonicalCommand> {
        if path.extension().and_then(|e| e.to_str()) != Some("c") {
            internal_error!("Gcc C Compiler received non-C file");
        }

        let mut cmd = CannonicalCommandBuilder::new("gcc");

        cmd.arg("-c").arg(path).arg("-o").arg(output);

        build_command_compile(path, cmd, profile, target, includes)
    }
}

impl CppCompiler for Gcc {
    fn new() -> Self
    where
        Self: Sized,
    {
        Gcc
    }
    fn get_dependencies(
        &self,
        src: &Path,
        profile: &crate::driver::runtime::Profile,
        target: &crate::driver::runtime::Target,
        includes: &[PathBuf],
    ) -> CoreResult<Vec<PathBuf>> {
        let mut cmd = CannonicalCommandBuilder::new("g++");
        cmd.arg("-MM").arg(src);
        let cmd = build_command_compile(src, cmd, profile, target, includes)?;
        let output = Command::from(&cmd).output()?;

        if !output.status.success() {
            return Err(Box::new(CoreError::ResolveDependency {
                err: String::from_utf8_lossy(&output.stderr).to_string(),
                cmd: display_command(&std::process::Command::from(&cmd)),
            }));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);

        let mut deps = Vec::new();
        for line in stdout.lines() {
            let line = line.trim_end_matches('\\').trim();
            if let Some(pos) = line.find(':') {
                let files = &line[pos + 1..];
                deps.extend(
                    files
                        .split_whitespace()
                        .map(|s| {
                            let path = PathBuf::from(s);
                            match path.canonicalize() {
                                Ok(p) => Ok(p),
                                Err(e) => {
                                    warn!(&format!(
                                        "Failed to cannonicalize dependency path: {}",
                                        path.display()
                                    ));
                                    Err(e)
                                }
                            }
                        })
                        .filter_map(|res| res.ok()),
                );
            }
        }

        Ok(deps)
    }
    fn compile_unit_cmd(
        &self,
        path: &std::path::Path,
        output: &std::path::Path,
        profile: &crate::driver::runtime::Profile,
        target: &crate::driver::runtime::Target,
        includes: &[PathBuf],
    ) -> crate::CoreResult<crate::driver::cannonical_command::CannonicalCommand> {
        if !matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("cpp") | Some("cxx") | Some("cc")
        ) {
            internal_error!("GCC C++ Compiler received non-C++ file");
        }

        let mut cmd = CannonicalCommandBuilder::new("g++");

        cmd.arg("-c").arg(path).arg("-o").arg(output);

        build_command_compile(path, cmd, profile, target, includes)
    }
}
