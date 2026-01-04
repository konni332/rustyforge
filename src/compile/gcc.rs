use std::path::PathBuf;

use anyhow::bail;

use crate::compile::{
    Compiler, object_path,
    types::{CannonicalCommand, CannonicalCommandBuilder, Profile},
};

pub struct Gcc;

impl Gcc {
    fn profile_flags(profile: Profile) -> &'static [&'static str] {
        match profile {
            Profile::Debug => &["-O0", "-g"],
            Profile::Release => &["-O2"],
        }
    }
}

impl Compiler for Gcc {
    fn compile_cmd(
        &self,
        unit: &super::types::CompileUnit,
        opts: &super::types::CompileOptions,
    ) -> anyhow::Result<CannonicalCommand> {
        let output = object_path(&opts.object_dir, unit.source)?;

        let mut cmd = CannonicalCommandBuilder::new("gcc");

        cmd.arg("-c").arg(unit.source).arg("-o").arg(output);

        cmd.args(Self::profile_flags(opts.profile));

        if let Some(target) = &opts.target {
            cmd.arg("--target").arg(target);
        }

        for include in unit.includes {
            cmd.arg("-I").arg(include);
        }

        for define in unit.defines {
            cmd.arg("-D").arg(define);
        }

        cmd.args(&opts.user_flags);

        Ok(cmd.finish())
    }
    fn get_dependencies(&self, file: &std::path::Path) -> anyhow::Result<Vec<std::path::PathBuf>> {
        let tmp_d = tempfile::NamedTempFile::new()?.into_temp_path();

        let status = std::process::Command::new("gcc")
            .arg("-MMD")
            .arg("-MF")
            .arg(&tmp_d)
            .arg(file)
            .status()?;

        if !status.success() {
            bail!("failed to generate dependency file for {}", file.display());
        }

        let content = std::fs::read_to_string(&tmp_d)?;
        let mut deps = Vec::new();
        for part in content.split_whitespace().skip(1) {
            let path = PathBuf::from(part.trim_end_matches('\\'));
            deps.push(path);
        }

        Ok(deps)
    }
    fn id(&self) -> &'static str {
        "gcc"
    }
}
