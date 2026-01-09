use std::fmt::Debug;

use serde::Serialize;

use crate::{
    CoreError, CoreResult, ToolConfig,
    config::tool_config::{
        ArchiverKind, CCompilerKind, CppCompilerKind, LinkerKind, ToolchainExecutable,
        ToolchainOption,
    },
    driver::runtime::RuntimeToolchain,
};

impl RuntimeToolchain {
    fn try_from_config(
        config_toolchain: &crate::config::tool_config::Toolchain,
        toolchain_label: &str,
    ) -> CoreResult<Self> {
        Ok(RuntimeToolchain {
            c_compiler: resolve_tool(
                config_toolchain.c_compiler.as_ref(),
                "c compiler",
                toolchain_label,
            )?,
            cpp_compiler: resolve_tool(
                config_toolchain.cpp_compiler.as_ref(),
                "c++ compiler",
                toolchain_label,
            )?,
            linker: resolve_tool(config_toolchain.linker.as_ref(), "linker", toolchain_label)?,
            archiver: resolve_tool(
                config_toolchain.archiver.as_ref(),
                "archiver",
                toolchain_label,
            )?,
        })
    }
}

pub fn resolve_toolchain(config: &ToolConfig) -> CoreResult<RuntimeToolchain> {
    if let Some(toolchains) = &config.toolchain {
        if let Some(platform) = toolchains.get(std::env::consts::OS) {
            RuntimeToolchain::try_from_config(platform, std::env::consts::OS)
        } else if let Some(default) = toolchains.get("default") {
            RuntimeToolchain::try_from_config(default, "default")
        } else {
            try_discover_toolchain()
        }
    } else {
        try_discover_toolchain()
    }
}

fn resolve_tool<T>(
    opt: Option<&ToolchainOption<T>>,
    tool_label: &str,
    toolchain_label: &str,
) -> CoreResult<T>
where
    T: Clone + Debug + Serialize + ToolchainExecutable,
{
    let candidates = match opt {
        Some(ToolchainOption::Single(t)) => vec![t.clone()],
        Some(ToolchainOption::List(list)) => list.clone(),
        None => {
            return try_discover_tool::<T>();
        }
    };

    for candidate in candidates.iter() {
        let exe = candidate.executable();
        if which::which(exe).is_ok() {
            return Ok(candidate.clone());
        }
    }

    Err(Box::new(CoreError::ToolChain {
        msg: format!("no suitable {tool_label} found for {toolchain_label}"),
        help: format!(
            "tried: {} - ensure at least one of them is installed and in PATH",
            candidates
                .iter()
                .map(|c| c.executable())
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }))
}

fn try_discover_tool<T: Clone + Debug + Serialize + ToolchainExecutable>() -> CoreResult<T> {
    T::discover()
}

fn try_discover_toolchain() -> CoreResult<RuntimeToolchain> {
    Ok(RuntimeToolchain {
        c_compiler: CCompilerKind::discover()?,
        cpp_compiler: CppCompilerKind::discover()?,
        linker: LinkerKind::discover()?,
        archiver: ArchiverKind::discover()?,
    })
}
