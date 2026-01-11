use crate::{
    TargetKind,
    driver::{
        cannonical_command::CannonicalCommandBuilder,
        toolchain::{
            PROFILE_DEFINE_TEMPLATE,
            traits::{CCompiler, CppCompiler},
        },
    },
    internal_error,
};

pub struct Intel;

fn intel_id() -> &'static str {
    "intel"
}

impl CCompiler for Intel {
    fn id(&self) -> &'static str {
        intel_id()
    }
    fn compile_unit_cmd(
        &self,
        path: &std::path::Path,
        output: &std::path::Path,
        profile: &crate::driver::runtime::Profile,
        target: &crate::driver::runtime::Target,
    ) -> crate::CoreResult<crate::driver::cannonical_command::CannonicalCommand> {
        if path.extension().and_then(|e| e.to_str()) != Some("c") {
            internal_error!("Intel C Compiler received non-C file");
        }

        let mut cmd = CannonicalCommandBuilder::new("icc");

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

impl CppCompiler for Intel {
    fn id(&self) -> &'static str {
        intel_id()
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
            internal_error!("Intel C++ Compiler received non-C++ file");
        }

        let mut cmd = CannonicalCommandBuilder::new("icpc");

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
