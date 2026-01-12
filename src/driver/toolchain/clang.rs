use std::{
    hash::Hash,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    CoreError, CoreResult, TargetKind,
    driver::{
        cannonical_command::{CannonicalCommand, CannonicalCommandBuilder},
        runtime::{Profile, Target},
        toolchain::{
            PROFILE_DEFINE_TEMPLATE,
            traits::{Archiver, CCompiler, CppCompiler, Linker},
        },
    },
    internal_error,
    utils::display_command,
    warn,
};

pub struct Clang;

fn clang_id() -> &'static str {
    "clang"
}

impl CCompiler for Clang {
    fn new() -> Self {
        Clang
    }
    fn get_dependencies(
        &self,
        src: &Path,
        profile: &crate::driver::runtime::Profile,
        target: &crate::driver::runtime::Target,
        includes: &[PathBuf],
    ) -> CoreResult<Vec<PathBuf>> {
        let mut cmd = CannonicalCommandBuilder::new("clang");
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
            internal_error!("Clang C Compiler received non-C file");
        }

        let mut cmd = CannonicalCommandBuilder::new("clang");

        cmd.arg("-c").arg(path).arg("-o").arg(output);

        build_command_compile(path, cmd, profile, target, includes)
    }
}

impl CppCompiler for Clang {
    fn new() -> Self {
        Clang
    }
    fn get_dependencies(
        &self,
        src: &Path,
        profile: &crate::driver::runtime::Profile,
        target: &crate::driver::runtime::Target,
        includes: &[PathBuf],
    ) -> CoreResult<Vec<PathBuf>> {
        let mut cmd = CannonicalCommandBuilder::new("clang++");
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
            internal_error!("Clang C++ Compiler received non-C++ file");
        }

        let mut cmd = CannonicalCommandBuilder::new("clang++");

        cmd.arg("-c").arg(path).arg("-o").arg(output);

        build_command_compile(path, cmd, profile, target, includes)
    }
}

pub fn build_command_compile(
    src: &std::path::Path,
    mut cmd: CannonicalCommandBuilder,
    profile: &Profile,
    target: &Target,
    includes: &[PathBuf],
) -> CoreResult<CannonicalCommand> {
    let opt = profile.opt_level.clamp(0, 3);
    cmd.arg(format!("-O{}", opt));

    if profile.debug {
        cmd.arg("-g");
    }

    if profile.lto {
        cmd.arg("-flto");
    }

    if matches!(target.kind, TargetKind::Shared) {
        cmd.arg("-fPIC");
    }

    for dir in includes {
        cmd.arg("-I").arg(dir);
    }

    cmd.arg(format!(
        "-D{}{}",
        PROFILE_DEFINE_TEMPLATE,
        profile.name.to_ascii_uppercase()
    ));

    if let Some(defines) = &target.defines {
        for def in defines {
            cmd.arg(format!("-D{}", def));
        }
    }

    if let Some(flags) = &profile.flags {
        cmd.args(flags);
    }

    Ok(cmd.finish())
}
