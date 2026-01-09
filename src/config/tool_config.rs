use std::{collections::HashMap, fmt::Debug, num::NonZero};

use serde::{Deserialize, Serialize};

use crate::{CoreError, CoreResult};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ToolConfig {
    pub build: Option<Build>,
    pub toolchain: Option<HashMap<String, Toolchain>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Build {
    pub threads: Option<NonZero<usize>>,
    pub compiler_flags: Option<Vec<String>>,
    pub linker_flags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Toolchain {
    #[serde(rename = "c")]
    pub c_compiler: Option<ToolchainOption<CCompilerKind>>,

    #[serde(rename = "cxx")]
    pub cpp_compiler: Option<ToolchainOption<CppCompilerKind>>,

    pub linker: Option<ToolchainOption<LinkerKind>>,

    #[serde(rename = "ar")]
    pub archiver: Option<ToolchainOption<ArchiverKind>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(bound(deserialize = "T: Deserialize<'de>"))]
pub enum ToolchainOption<T: Debug + Clone + Serialize> {
    Single(T),
    List(Vec<T>),
}

pub trait ToolchainExecutable: Sized + Debug + Serialize {
    fn executable(&self) -> &'static str;
    fn discover() -> CoreResult<Self>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ArchiverKind {
    #[serde(rename = "ar")]
    Ar, // Unix/MacOS ar
    #[serde(rename = "lib.exe")]
    Lib, // MSVC lib.exe
    #[serde(rename = "llvm-ar")]
    LlvmAr,
}

impl ToolchainExecutable for ArchiverKind {
    fn executable(&self) -> &'static str {
        match self {
            ArchiverKind::Ar => "ar",
            ArchiverKind::Lib => "lib.exe",
            ArchiverKind::LlvmAr => {
                if cfg!(target_os = "windows") {
                    "llvm-ar.exe"
                } else {
                    "llvm-ar"
                }
            }
        }
    }
    fn discover() -> CoreResult<Self> {
        #[cfg(target_os = "windows")]
        let candidates = [Self::Lib, Self::LlvmAr];

        #[cfg(not(target_os = "windows"))]
        let candidates = [Self::LlvmAr, Self::Ar];

        for candidate in candidates.iter() {
            if which::which(candidate.executable()).is_ok() {
                return Ok(candidate.clone());
            }
        }

        Err(Box::new(CoreError::ToolChain {
            msg: "no suitable archiver found".into(),
            help: format!(
                "tried: {}",
                candidates
                    .iter()
                    .map(|c| c.executable())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CCompilerKind {
    #[serde(rename = "gcc")]
    Gcc,
    #[serde(rename = "clang")]
    Clang,
    #[serde(rename = "cl.exe")]
    Msvc, // Microsoft Visual C++
    #[serde(rename = "icc")]
    Icc, // Intel C Compiler
}

impl ToolchainExecutable for CCompilerKind {
    fn executable(&self) -> &'static str {
        match self {
            CCompilerKind::Gcc => {
                if cfg!(target_os = "windows") {
                    "gcc.exe"
                } else {
                    "gcc"
                }
            }
            CCompilerKind::Clang => {
                if cfg!(target_os = "windows") {
                    "clang.exe"
                } else {
                    "clang"
                }
            }
            CCompilerKind::Msvc => "cl.exe",
            CCompilerKind::Icc => {
                if cfg!(target_os = "windows") {
                    "icc.exe"
                } else {
                    "icc"
                }
            }
        }
    }

    fn discover() -> CoreResult<Self> {
        #[cfg(target_os = "windows")]
        let candidates = [Self::Msvc, Self::Clang, Self::Gcc, Self::Icc];

        #[cfg(not(target_os = "windows"))]
        let candidates = [Self::Clang, Self::Gcc, Self::Icc];

        for candidate in candidates.iter() {
            if which::which(candidate.executable()).is_ok() {
                return Ok(candidate.clone());
            }
        }

        Err(Box::new(CoreError::ToolChain {
            msg: "no suitable C compiler found".into(),
            help: format!(
                "tried: {}",
                candidates
                    .iter()
                    .map(|c| c.executable())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }))
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CppCompilerKind {
    #[serde(rename = "g++")]
    Gpp,
    #[serde(rename = "clang++")]
    Clangpp,
    #[serde(rename = "cl.exe")]
    Msvc, // Microsoft Visual C++
    #[serde(rename = "icpc")]
    Icc, // Intel C++ Compiler
}

impl ToolchainExecutable for CppCompilerKind {
    fn executable(&self) -> &'static str {
        match self {
            CppCompilerKind::Gpp => {
                if cfg!(target_os = "windows") {
                    "g++.exe"
                } else {
                    "g++"
                }
            }
            CppCompilerKind::Clangpp => {
                if cfg!(target_os = "windows") {
                    "clang++.exe"
                } else {
                    "clang++"
                }
            }
            CppCompilerKind::Msvc => "cl.exe",
            CppCompilerKind::Icc => {
                if cfg!(target_os = "windows") {
                    "icpc.exe"
                } else {
                    "icpc"
                }
            }
        }
    }
    fn discover() -> CoreResult<Self> {
        #[cfg(target_os = "windows")]
        let candidates = [Self::Msvc, Self::Clangpp, Self::Gpp, Self::Icc];

        #[cfg(not(target_os = "windows"))]
        let candidates = [Self::Clangpp, Self::Gpp, Self::Icc];

        for candidate in candidates.iter() {
            if which::which(candidate.executable()).is_ok() {
                return Ok(candidate.clone());
            }
        }

        Err(Box::new(CoreError::ToolChain {
            msg: "no suitable C++ compiler found".into(),
            help: format!(
                "tried: {}",
                candidates
                    .iter()
                    .map(|c| c.executable())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LinkerKind {
    #[serde(rename = "ld")]
    Ld,
    #[serde(rename = "clang")]
    Clang,
    #[serde(rename = "link.exe")]
    Msvc,
    #[serde(rename = "lld")]
    Lld,
}

impl ToolchainExecutable for LinkerKind {
    fn executable(&self) -> &'static str {
        match self {
            LinkerKind::Ld => {
                if cfg!(target_os = "windows") {
                    "ld.exe"
                } else {
                    "ld"
                }
            }
            LinkerKind::Clang => {
                if cfg!(target_os = "windows") {
                    "clang.exe"
                } else {
                    "clang"
                }
            }
            LinkerKind::Msvc => "link.exe",
            LinkerKind::Lld => {
                if cfg!(target_os = "windows") {
                    "lld.exe"
                } else {
                    "lld"
                }
            }
        }
    }
    fn discover() -> CoreResult<Self> {
        #[cfg(target_os = "windows")]
        let candidates = [Self::Msvc, Self::Lld];

        #[cfg(not(target_os = "windows"))]
        let candidates = [Self::Lld, Self::Ld, Self::Clang];

        for candidate in candidates.iter() {
            if which::which(candidate.executable()).is_ok() {
                return Ok(candidate.clone());
            }
        }

        Err(Box::new(CoreError::ToolChain {
            msg: "no suitable linker found".into(),
            help: format!(
                "tried: {}",
                candidates
                    .iter()
                    .map(|c| c.executable())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }))
    }
}
