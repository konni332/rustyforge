use anyhow::{Result, ensure};

use crate::{
    compile::types::{CannonicalCommand, CannonicalCommandBuilder},
    config::project::LinkTargetKind,
    link::{
        Linker,
        types::{LinkOptions, LinkUnit},
    },
};

pub struct ClangLinker;

impl Linker for ClangLinker {
    fn new() -> Self {
        ClangLinker
    }
    fn link_cmd(
        &self,
        unit: &super::types::LinkUnit,
        opts: &super::types::LinkOptions,
    ) -> anyhow::Result<crate::compile::types::CannonicalCommand> {
        ensure!(!unit.objects.is_empty(), "no object files to link");
        match unit.kind {
            LinkTargetKind::Executable => {
                ensure!(
                    unit.output.extension() != Some("a".as_ref()),
                    "executable output cannot be .a"
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
            LinkTargetKind::Executable => self.link_executable(unit, opts),
            LinkTargetKind::StaticLibrary => self.link_static_lib(unit),
            LinkTargetKind::SharedLibrary => self.link_shared_lib(unit, opts),
        }
    }
}

impl ClangLinker {
    fn link_executable(&self, unit: &LinkUnit, opts: &LinkOptions) -> Result<CannonicalCommand> {
        let mut cmd = CannonicalCommandBuilder::new("clang");

        if let Some(target) = &opts.target {
            cmd.arg("--target").arg(target);
        }

        for obj in unit.objects {
            cmd.arg(obj);
        }

        for dir in &unit.lib_dirs {
            cmd.arg("-L").arg(dir);
        }

        for lib in &unit.libs {
            cmd.arg(format!("-l{}", lib));
        }

        cmd.arg("-o").arg(unit.output);
        cmd.args(unit.user_flags);

        Ok(cmd.finish())
    }
    fn link_static_lib(&self, unit: &LinkUnit) -> Result<CannonicalCommand> {
        let mut cmd = CannonicalCommandBuilder::new("ar");

        cmd.arg("rcs");
        cmd.arg(unit.output);

        for obj in unit.objects {
            cmd.arg(obj);
        }

        cmd.args(unit.user_flags);

        Ok(cmd.finish())
    }
    fn link_shared_lib(&self, unit: &LinkUnit, opts: &LinkOptions) -> Result<CannonicalCommand> {
        let mut cmd = CannonicalCommandBuilder::new("clang");

        if let Some(target) = &opts.target {
            cmd.arg("--target").arg(target);
        }

        for obj in unit.objects {
            cmd.arg(obj);
        }

        for dir in &unit.lib_dirs {
            cmd.arg("-L").arg(dir);
        }

        for lib in &unit.libs {
            cmd.arg(format!("-l{}", lib));
        }

        cmd.arg("-shared");

        cmd.arg("-o").arg(unit.output);

        cmd.args(unit.user_flags);

        Ok(cmd.finish())
    }
}
