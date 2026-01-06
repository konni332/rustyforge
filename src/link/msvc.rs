use anyhow::{Result, ensure};

use crate::{
    compile::types::{CannonicalCommand, CannonicalCommandBuilder},
    config::project::LinkTargetKind,
    link::{Linker, types::LinkUnit},
};

pub struct MsvcLinker;

impl Linker for MsvcLinker {
    fn new() -> Self {
        MsvcLinker
    }
    fn link_cmd(
        &self,
        unit: &super::types::LinkUnit,
        _opts: &super::types::LinkOptions, // No crosscompiling with msvc
    ) -> anyhow::Result<crate::compile::types::CannonicalCommand> {
        ensure!(!unit.objects.is_empty(), "no object files to link");
        match unit.kind {
            LinkTargetKind::Executable => {
                ensure!(
                    unit.output.extension() == Some("exe".as_ref()),
                    "executable must end with .exe"
                );
            }
            LinkTargetKind::StaticLibrary => {
                ensure!(
                    unit.output.extension() == Some("a".as_ref()),
                    "static library must end with .a"
                );
            }
            _ => {}
        }

        match unit.kind {
            LinkTargetKind::Executable => self.link_executable(unit),
            LinkTargetKind::StaticLibrary => self.link_static_lib(unit),
            LinkTargetKind::SharedLibrary => self.link_shared_lib(unit),
        }
    }
}

impl MsvcLinker {
    fn link_executable(&self, unit: &LinkUnit) -> Result<CannonicalCommand> {
        let mut cmd = CannonicalCommandBuilder::new("link.exe");

        for obj in unit.objects {
            cmd.arg(obj);
        }

        cmd.arg("/OUT:").arg(&unit.output);

        for dir in &unit.lib_dirs {
            cmd.arg(format!("/LIBPATH:{}", dir.display()));
        }

        for lib in &unit.libs {
            cmd.arg(format!("{}.lib", lib));
        }

        cmd.args(unit.user_flags);

        Ok(cmd.finish())
    }
    fn link_static_lib(&self, unit: &LinkUnit) -> Result<CannonicalCommand> {
        let mut cmd = CannonicalCommandBuilder::new("lib.exe");

        cmd.arg(format!("/OUT:{}", unit.output.display()));

        for obj in unit.objects {
            cmd.arg(obj);
        }

        cmd.args(unit.user_flags);

        Ok(cmd.finish())
    }
    fn link_shared_lib(&self, unit: &LinkUnit) -> Result<CannonicalCommand> {
        let mut cmd = CannonicalCommandBuilder::new("link.exe");

        for obj in unit.objects {
            cmd.arg(obj);
        }

        cmd.arg("/DLL");
        cmd.arg(format!("/OUT:{}", unit.output.display()));

        for dir in &unit.lib_dirs {
            cmd.arg(format!("/LIBPATH:{}", dir.display()));
        }

        for lib in &unit.libs {
            cmd.arg(format!("{}.lib", lib));
        }

        cmd.args(unit.user_flags);

        Ok(cmd.finish())
    }
}
