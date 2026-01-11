use std::path::PathBuf;

use crate::{
    CoreError, CoreResult, TargetKind,
    driver::{
        cannonical_command::CannonicalCommandBuilder,
        toolchain::{
            PROFILE_DEFINE_TEMPLATE,
            traits::{Archiver, CCompiler, CppCompiler, Linker},
        },
    },
    internal_error, warn,
};

pub struct Gcc;

fn gcc_id() -> &'static str {
    "gcc"
}

impl CCompiler for Gcc {
    fn get_dependencies(
        &self,
        src: &std::path::Path,
    ) -> crate::CoreResult<Vec<std::path::PathBuf>> {
        get_gcc_dependencies(src, "gcc")
    }
    fn id(&self) -> &'static str {
        gcc_id()
    }
    fn compile_unit_cmd(
        &self,
        path: &std::path::Path,
        output: &std::path::Path,
        profile: &crate::driver::runtime::Profile,
        target: &crate::driver::runtime::Target,
    ) -> crate::CoreResult<crate::driver::cannonical_command::CannonicalCommand> {
        if path.extension().and_then(|e| e.to_str()) != Some("c") {
            internal_error!("Gcc C Compiler received non-C file");
        }

        let mut cmd = CannonicalCommandBuilder::new("gcc");

        cmd.arg("-c").arg(path).arg("-o").arg(output);

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
}

impl CppCompiler for Gcc {
    fn get_dependencies(
        &self,
        src: &std::path::Path,
    ) -> crate::CoreResult<Vec<std::path::PathBuf>> {
        get_gcc_dependencies(src, "g++")
    }
    fn id(&self) -> &'static str {
        gcc_id()
    }
    fn compile_unit_cmd(
        &self,
        path: &std::path::Path,
        output: &std::path::Path,
        profile: &crate::driver::runtime::Profile,
        target: &crate::driver::runtime::Target,
    ) -> crate::CoreResult<crate::driver::cannonical_command::CannonicalCommand> {
        if !matches!(
            path.extension().and_then(|e| e.to_str()),
            Some("cpp") | Some("cxx") | Some("cc")
        ) {
            internal_error!("GCC C++ Compiler received non-C++ file");
        }

        let mut cmd = CannonicalCommandBuilder::new("g++");

        cmd.arg("-c").arg(path).arg("-o").arg(output);

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
}

impl Linker for Gcc {
    fn id(&self) -> &'static str {
        gcc_id()
    }
}

impl Archiver for Gcc {
    fn id(&self) -> &'static str {
        gcc_id()
    }
}

fn get_gcc_dependencies(src: &std::path::Path, exe: &str) -> CoreResult<Vec<std::path::PathBuf>> {
    use std::io::{self, BufRead};
    use std::process::Command;

    let output = Command::new(exe).arg("-MM").arg(src).output()?;

    if !output.status.success() {
        return Err(Box::new(CoreError::ResolveDependency {
            path: src.to_path_buf(),
            err: String::from_utf8_lossy(&output.stderr).to_string(),
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
