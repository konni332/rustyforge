use std::path::{Path, PathBuf};

use crate::{
    CoreResult, TargetKind,
    driver::{
        cannonical_command::{CannonicalCommand, CannonicalCommandBuilder},
        runtime::Profile,
        toolchain::{
            PROFILE_DEFINE_TEMPLATE,
            traits::{Archiver, CCompiler, CppCompiler, Linker},
        },
    },
    internal_error, warn,
};

pub struct Msvc;

impl CCompiler for Msvc {
    fn get_dependencies(
        &self,
        src: &Path,
        profile: &crate::driver::runtime::Profile,
        target: &crate::driver::runtime::Target,
        includes: &[PathBuf],
    ) -> CoreResult<Vec<PathBuf>> {
        Ok(vec![])
    }
    fn new() -> Self
    where
        Self: Sized,
    {
        Msvc
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
            internal_error!("Msvc C Compiler received non-C file");
        }

        let mut cmd = CannonicalCommandBuilder::new("cl.exe");

        cmd.arg("/c")
            .arg("/TC")
            .arg(path)
            .arg(format!("/Fo:{}", output.display()));

        let opt_flag = match profile.opt_level {
            0 => "/Od",
            1 => "/O1",
            _ => "/O2",
        };
        cmd.arg(opt_flag);

        if profile.debug {
            cmd.arg("/Zi");
        }

        for dir in includes {
            cmd.arg("/I").arg(dir);
        }

        cmd.arg(format!(
            "/D{}{}",
            PROFILE_DEFINE_TEMPLATE,
            profile.name.to_ascii_uppercase()
        ));

        if let Some(defines) = &target.defines {
            for def in defines {
                cmd.arg(format!("/D{}", def));
            }
        }

        if let Some(flags) = &profile.flags {
            cmd.args(flags);
        }

        Ok(cmd.finish())
    }
}

impl CppCompiler for Msvc {
    fn new() -> Self
    where
        Self: Sized,
    {
        Msvc
    }
    fn get_dependencies(
        &self,
        src: &Path,
        profile: &crate::driver::runtime::Profile,
        target: &crate::driver::runtime::Target,
        includes: &[PathBuf],
    ) -> CoreResult<Vec<PathBuf>> {
        Ok(vec![])
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
            internal_error!("Msvc C++ Compiler received non-C++ file");
        }

        let mut cmd = CannonicalCommandBuilder::new("cl.exe");

        cmd.arg("/c")
            .arg("/TP")
            .arg(path)
            .arg(format!("/Fo:{}", output.display()));

        let opt_flag = match profile.opt_level {
            0 => "/Od",
            1 => "/O1",
            _ => "/O2",
        };
        cmd.arg(opt_flag);

        if profile.debug {
            cmd.arg("/Zi");
        }

        for dir in includes {
            cmd.arg("/I").arg(dir);
        }

        cmd.arg(format!(
            "/D{}{}",
            PROFILE_DEFINE_TEMPLATE,
            profile.name.to_ascii_uppercase()
        ));

        if let Some(defines) = &target.defines {
            for def in defines {
                cmd.arg(format!("/D{}", def));
            }
        }

        if let Some(flags) = &profile.flags {
            cmd.args(flags);
        }

        Ok(cmd.finish())
    }
}

/// Unfortunatly Msvc pretty much compiles the src to fetch dependencies, so a build cache in the
/// compilation layer does not save any time. All Msvc builds ignore the build cache entirely!
fn get_msvc_dependencies() -> CoreResult<Vec<PathBuf>> {
    Ok(vec![])
}

impl Linker for Msvc {
    fn new() -> Self
    where
        Self: Sized,
    {
        Msvc
    }
    fn link_objects(
        &self,
        target: &crate::driver::runtime::Target,
        profile: &Profile,
        objs: &[PathBuf],
        contains_cpp: bool,
        lib_dirs: &[PathBuf],
        output: &Path,
    ) -> CoreResult<CannonicalCommand> {
        if target.kind == TargetKind::Static {
            internal_error!(
                "Tried to link static library target, should have used archiver instead"
            );
        }

        let mut cmd = CannonicalCommandBuilder::new("cl.exe");

        if profile.lto {
            cmd.arg("/GL");
        }

        for obj in objs {
            cmd.arg(obj);
        }

        cmd.arg("/link");

        if matches!(target.kind, TargetKind::Shared) {
            cmd.arg("/DLL");
        }

        if profile.lto {
            cmd.arg("/LTCG");
        }

        for dir in lib_dirs {
            cmd.arg(format!("/LIBPATH:{}", dir.display()));
        }

        cmd.arg(format!("/OUT:{}", output.display()));

        if let Some(flags) = &target.flags {
            cmd.args(flags);
        }

        Ok(cmd.finish())
    }
}
