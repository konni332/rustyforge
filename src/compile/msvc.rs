use std::path::PathBuf;

use anyhow::bail;

use crate::compile::{
    Compiler, object_path,
    types::{CannonicalCommand, CannonicalCommandBuilder, Profile},
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

        for define in unit.defines {
            cmd.arg("/D").arg(define);
        }

        cmd.args(&opts.user_flags);

        if let Some(target) = &opts.target {
            cmd.arg(format!("/arch:{}", target));
        }

        Ok(cmd.finish())
    }
    fn get_dependencies(&self, file: &std::path::Path) -> anyhow::Result<Vec<std::path::PathBuf>> {
        let output = std::process::Command::new("cl.exe")
            .arg("/nologo")
            .arg("/showIncludes")
            .arg("/c")
            .arg(file)
            .output()?;

        if !output.status.success() {
            bail!("failed to get dependencies for {}", file.display());
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
    fn id(&self) -> &'static str {
        "msvc"
    }
}
