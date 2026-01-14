use serde::Serialize;

use crate::{
    CoreResult, RunTimeConfig, TargetKind, ToolchainExecutable,
    driver::runtime::{Meta, Profile, RuntimeToolchain, Target},
};
use std::{fmt::Write, path::PathBuf};

#[derive(Debug, Serialize)]
pub struct ProjectInfo<'i> {
    pub config: &'i RunTimeConfig<'i>,
    pub includes: Option<Vec<PathBuf>>,
    pub c_sources: Option<Vec<PathBuf>>,
    pub cpp_sources: Option<Vec<PathBuf>>,
}

use std::fmt;

impl<'i> fmt::Display for ProjectInfo<'i> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cfg = self.config;

        writeln!(f, "{}", meta_info(&cfg.meta))?;
        writeln!(f, "{}", profile_info(&cfg.profile))?;
        writeln!(f, "{}", target_info(&cfg.targets))?;
        writeln!(f, "{}", toolchain_info(&cfg.toolchain))?;

        if let Some(includes) = &self.includes {
            writeln!(f, "\nIncludes:")?;
            for inc in includes {
                writeln!(f, "  - {}", inc.display())?;
            }
        }

        if let Some(c_sources) = &self.c_sources {
            writeln!(f, "\nC Sources:")?;
            for src in c_sources {
                writeln!(f, "  - {}", src.display())?;
            }
        }

        if let Some(cpp_sources) = &self.cpp_sources {
            writeln!(f, "\nC++ Sources:")?;
            for src in cpp_sources {
                writeln!(f, "  - {}", src.display())?;
            }
        }

        Ok(())
    }
}

fn meta_info(meta: &Meta) -> String {
    format!(
        "\nProject Meta:\n  Name: {}\n  Version: {}\n  C Edition: {}\n  Cpp Edition: {}\n  Threads: {}",
        meta.name, meta.version, meta.c_edition, meta.cpp_edition, meta.threads
    )
}

fn profile_info(profile: &Profile) -> String {
    format!(
        "\nProfile:\n  Name: {}\n  Flags: {}\n  LTO: {}\n  Opt Level: {}\n  Debug: {}",
        profile.name,
        profile
            .flags
            .as_ref()
            .map(|f| f.join(" "))
            .unwrap_or_else(|| "-".into()),
        profile.lto,
        profile.opt_level,
        profile.debug,
    )
}

fn target_info(targets: &[Target]) -> String {
    let mut s = String::from("\nTargets:");
    for target in targets {
        let kind = match &target.kind {
            TargetKind::Static => "Static Libarry".to_string(),
            TargetKind::Shared => "Shared Library".to_string(),
            TargetKind::Executable { entry } => format!("Executable (entry: {})", entry.display()),
        };

        let flags = target
            .flags
            .as_ref()
            .map(|f| f.join(" "))
            .unwrap_or_else(|| "-".into());
        let ignore = target
            .ignore
            .as_ref()
            .map(|i| i.join(" "))
            .unwrap_or_else(|| "-".into());
        let defines = target
            .defines
            .as_ref()
            .map(|d| d.join(" "))
            .unwrap_or_else(|| "-".into());

        s.push_str(&format!(
            "\n  - Name: {}\n    Kind: {}\n    Flags: {}\n    Ingore: {}\n    Defines: {}\n",
            target.name, kind, flags, ignore, defines
        ));
    }

    s
}

fn toolchain_info(toolchain: &RuntimeToolchain) -> String {
    format!(
        "Toolchain:\n  C Compiler: {}\n  C++ Compiler: {}\n  Linker: {}\n  Archiver: {}",
        toolchain.c_compiler.executable(),
        toolchain.cpp_compiler.executable(),
        toolchain.linker.executable(),
        toolchain.archiver.executable(),
    )
}
