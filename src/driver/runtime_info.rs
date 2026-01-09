use crate::{
    CoreResult, RunTimeConfig, TargetKind, ToolchainExecutable,
    driver::runtime::{Meta, Profile, RuntimeToolchain, Target},
};
use std::fmt::Write;

pub fn get_project_info_string(runtime_config: &RunTimeConfig) -> CoreResult<String> {
    let mut info = String::new();

    writeln!(info, "{}", meta_info(&runtime_config.meta))?;
    writeln!(info, "{}", profile_info(&runtime_config.profile))?;
    writeln!(info, "{}", target_info(&runtime_config.targets))?;
    writeln!(info, "{}", toolchain_info(&runtime_config.toolchain))?;

    Ok(info)
}

fn meta_info(meta: &Meta) -> String {
    format!(
        "\nProject Meta:\n  Name: {}\n  Version: {}\n  Edition: {}\n  Threads: {}",
        meta.name, meta.version, meta.edition, meta.threads
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
