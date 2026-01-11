use crate::{
    driver::{
        cannonical_command::CannonicalCommandBuilder,
        toolchain::{
            PROFILE_DEFINE_TEMPLATE,
            traits::{Archiver, CCompiler, CppCompiler, Linker},
        },
    },
    internal_error,
};

pub struct Msvc;

fn msvc_id() -> &'static str {
    "msvc"
}

impl CCompiler for Msvc {
    fn id(&self) -> &'static str {
        msvc_id()
    }
    fn compile_unit_cmd(
        &self,
        path: &std::path::Path,
        output: &std::path::Path,
        profile: &crate::driver::runtime::Profile,
        target: &crate::driver::runtime::Target,
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
    fn id(&self) -> &'static str {
        msvc_id()
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

impl Linker for Msvc {
    fn id(&self) -> &'static str {
        msvc_id()
    }
}

impl Archiver for Msvc {
    fn id(&self) -> &'static str {
        msvc_id()
    }
}
