use std::fmt::Debug;

use serde::Serialize;

use crate::{
    CoreError, CoreResult, ToolConfig,
    config::tool_config::{
        ArchiverKind, CCompilerKind, CppCompilerKind, LinkerKind, ToolchainExecutable,
        ToolchainOption,
    },
    driver::runtime::RuntimeToolchain,
    warn,
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
    pub fn validate(&self) -> CoreResult<()> {
        if !Self::is_compatible_c_cpp(self.c_compiler, self.cpp_compiler) {
            return Err(Box::new(CoreError::ToolChain {
                msg: format!(
                    "Incompatible C/C++ compiler combination: {:?} + {:?}",
                    self.c_compiler, self.cpp_compiler
                ),
                help: "Use compilers from the same family for LTO builds.".to_string(),
            }));
        }

        if !Self::is_compatible_c_linker(self.c_compiler, self.linker) {
            return Err(Box::new(CoreError::ToolChain {
                msg: format!(
                    "C compiler and linker are incompatible: {:?} + {:?}",
                    self.c_compiler, self.linker
                ),
                help: "Ensure the linker matches the C compiler.".to_string(),
            }));
        }

        if !Self::is_compatible_cpp_linker(self.cpp_compiler, self.linker) {
            return Err(Box::new(CoreError::ToolChain {
                msg: format!(
                    "C++ compiler and linker are incompatible: {:?} + {:?}",
                    self.cpp_compiler, self.linker
                ),
                help: "Ensure the linker matches the C++ compiler.".to_string(),
            }));
        }

        if !Self::is_same_family(self.c_compiler, self.cpp_compiler) {
            warn!(
                &"Toolchain",
                &"C and C++ compilers are from different families. LTO builds may fail."
            );
        }

        if !Self::is_same_family_compiler_linker(self.c_compiler, self.cpp_compiler, self.linker) {
            warn!(
                &"Toolchain",
                &"Compiler and linker families differ. LTO builds may fail."
            );
        }

        Ok(())
    }

    fn is_compatible_c_cpp(c: CCompilerKind, cpp: CppCompilerKind) -> bool {
        Self::compiler_family_c(c) == Self::compiler_family_cpp(cpp)
    }

    fn is_compatible_c_linker(c: CCompilerKind, linker: LinkerKind) -> bool {
        Self::compiler_family_c(c) == Self::linker_family(linker)
    }

    fn is_compatible_cpp_linker(cpp: CppCompilerKind, linker: LinkerKind) -> bool {
        Self::compiler_family_cpp(cpp) == Self::linker_family(linker)
    }

    fn is_same_family(c: CCompilerKind, cpp: CppCompilerKind) -> bool {
        Self::compiler_family_c(c) == Self::compiler_family_cpp(cpp)
    }

    fn is_same_family_compiler_linker(
        c: CCompilerKind,
        cpp: CppCompilerKind,
        linker: LinkerKind,
    ) -> bool {
        let family = Self::linker_family(linker);
        Self::compiler_family_c(c) == family && Self::compiler_family_cpp(cpp) == family
    }

    fn compiler_family_c(c: CCompilerKind) -> &'static str {
        match c {
            CCompilerKind::Gcc => "gcc",
            CCompilerKind::Clang => "clang",
            CCompilerKind::Msvc => "msvc",
            CCompilerKind::Icc => "intel",
        }
    }
    fn compiler_family_cpp(cpp: CppCompilerKind) -> &'static str {
        match cpp {
            CppCompilerKind::Gpp => "gcc",
            CppCompilerKind::Clangpp => "clang",
            CppCompilerKind::Msvc => "msvc",
            CppCompilerKind::Icpc => "intel",
        }
    }

    fn linker_family(linker: LinkerKind) -> &'static str {
        match linker {
            LinkerKind::Gcc => "gcc",
            LinkerKind::Clang => "clang",
            LinkerKind::Msvc => "msvc",
            LinkerKind::Lld => "clang",
        }
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
    T: Clone + Copy + Debug + Serialize + ToolchainExecutable,
{
    let candidates = match opt {
        Some(ToolchainOption::Single(t)) => vec![*t],
        Some(ToolchainOption::List(list)) => list.clone(),
        None => {
            return try_discover_tool::<T>();
        }
    };

    for candidate in candidates.iter() {
        let exe = candidate.executable();
        if which::which(exe).is_ok() {
            return Ok(*candidate);
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
