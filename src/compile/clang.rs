use std::{path::PathBuf, process::Command};

use anyhow::bail;

use crate::{
    compile::{
        Compiler, object_path,
        types::{CannonicalCommand, CannonicalCommandBuilder, CompileUnit, Profile},
    },
    ui::output_error_compile,
};

pub struct Clang;

impl Clang {
    fn profile_flags(profile: Profile) -> &'static [&'static str] {
        match profile {
            Profile::Debug => &["-O0", "-g"],
            Profile::Release => &["-O2"],
        }
    }
}

impl Compiler for Clang {
    fn new() -> Self {
        Clang
    }
    fn compile_cmd(
        &self,
        unit: &super::types::CompileUnit,
        opts: &super::types::CompileOptions,
    ) -> anyhow::Result<CannonicalCommand> {
        let output = object_path(&opts.object_dir, unit.source)?;

        let mut cmd = CannonicalCommandBuilder::new("clang");

        cmd.arg("-c").arg(unit.source).arg("-o").arg(output);

        cmd.args(Self::profile_flags(opts.profile));

        if let Some(target) = &opts.target {
            cmd.arg("--target").arg(target);
        }

        for include in unit.includes {
            cmd.arg("-I").arg(include);
        }

        for define in unit.defines {
            cmd.arg(format!("-D{}", define));
        }

        cmd.args(&opts.user_flags);

        Ok(cmd.finish())
    }
    fn get_dependencies(&self, unit: &CompileUnit) -> anyhow::Result<Vec<std::path::PathBuf>> {
        let tmp_d = tempfile::NamedTempFile::new()?.into_temp_path();

        let mut cmd = Command::new("clang");
        cmd.arg("-MM")
            .arg("-MP")
            .arg("-MF")
            .arg(&tmp_d)
            .arg(unit.source);

        for include in unit.includes {
            cmd.arg("-I").arg(include);
        }

        for define in unit.defines {
            cmd.arg(format!("-D{}", define));
        }

        let output = cmd.output()?;

        if !output.status.success() {
            output_error_compile(unit.source, &cmd, &output.stderr);
            bail!("Dependency collection failed");
        }

        let content = std::fs::read_to_string(&tmp_d)?;
        let mut deps = Vec::new();
        for part in content.split_whitespace().skip(1) {
            let path = PathBuf::from(part.trim_end_matches('\\').trim_end_matches(':'));
            deps.push(path);
        }

        Ok(deps)
    }
}
