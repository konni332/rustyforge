use clap::ValueEnum;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ToolConfig {
    pub compiler: Option<CompilerKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize, ValueEnum)]
pub enum CompilerKind {
    Gcc,
    Clang,
    Msvc,
}

impl ToolConfig {
    pub fn new() -> Self {
        Self { compiler: None }
    }
}
