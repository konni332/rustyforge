use std::{path::PathBuf, process::Command};

use anyhow::bail;

use crate::{
    compile::{
        Compiler, object_path,
        types::{CannonicalCommand, CannonicalCommandBuilder, Profile},
    },
    ui::error_compile_msg,
};

pub struct Msvc;

impl Msvc {
    fn profile_flags(profile: Profile) -> &'static [&'static str] {
        match profile {
            Profile::Debug => &["/Od", "/Zi"],
            Profile::Release => &["/O2"],
        }
    }
}

impl Compiler for Msvc {
    fn new() -> Self {
        Msvc
    }
    fn compile_cmd(
        &self,
        unit: &super::types::CompileUnit,
        opts: &super::types::CompileOptions,
    ) -> anyhow::Result<CannonicalCommand> {
        let output = object_path(&opts.object_dir, unit.source)?;

        let mut cmd = CannonicalCommandBuilder::new("cl.exe");

        cmd.arg("/c").arg(unit.source).arg("/Fo").arg(output);

        cmd.args(Self::profile_flags(opts.profile));

        for include in unit.includes {
            cmd.arg("/I").arg(include);
        }

        if unit.is_shared {
            cmd.arg("/D").arg("BUILDING_MYLIB");
        }

        for define in unit.defines {
            cmd.arg("/D").arg(define);
        }

        cmd.args(&opts.user_flags);

        if let Some(target) = &opts.target {
            cmd.arg(format!("/arch:{}", target));
        }

        Ok(cmd.finish())
    }
    fn get_dependencies(
        &self,
        unit: &super::types::CompileUnit,
    ) -> anyhow::Result<Vec<std::path::PathBuf>> {
        let mut cmd = Command::new("cl.exe");
        cmd.arg("/nologo")
            .arg("/showIncludes")
            .arg("/c")
            .arg(unit.source)
            .output()?;

        for include in unit.includes {
            cmd.arg("/I").arg(include);
        }

        for define in unit.defines {
            cmd.arg("/D").arg(define);
        }

        let output = cmd.output()?;

        if !output.status.success() {
            error_compile_msg(unit.source, &cmd, &output.stderr);
            bail!("Dependency collection failed");
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut deps = vec![];

        for line in stdout.lines() {
            if let Some(rest) = line.strip_prefix("Note: including file: ") {
                deps.push(PathBuf::from(rest.trim()));
            }
        }

        Ok(deps)
    }
}
