use crate::config::tool::CompilerKind;
use anyhow::{Result, bail};

pub fn resolve_compiler(
    tool_config_compiler: Option<CompilerKind>,
    profile_config_compiler: Option<CompilerKind>,
) -> Result<CompilerKind> {
    let preferred = profile_config_compiler.or(tool_config_compiler);

    if let Some(compiler) = preferred
        && is_compiler_usable(compiler)
    {
        return Ok(compiler);
    }

    #[cfg(unix)]
    {
        if is_usable("clang") {
            return Ok(CompilerKind::Clang);
        }
        if is_usable("gcc") {
            return Ok(CompilerKind::Gcc);
        }
    }
    #[cfg(target_os = "windows")]
    if is_usable("cl.exe") {
        return Ok(CompilerKind::Msvc);
    }

    bail!("No suitable compiler found");
}

fn is_compiler_usable(kind: CompilerKind) -> bool {
    match kind {
        CompilerKind::Clang => is_usable("clang"),
        CompilerKind::Gcc => is_usable("gcc"),
        CompilerKind::Msvc => is_usable("cl.exe"),
    }
}

fn is_usable(cmd: &str) -> bool {
    which::which(cmd).is_ok()
}
