use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::compile::types::{CannonicalCommand, CompileOptions, CompileUnit};

mod clang;
mod driver;
mod gcc;
mod msvc;
pub mod types;

pub use clang::Clang;
pub use driver::CompilerDriver;
pub use gcc::Gcc;
pub use msvc::Msvc;

pub trait Compiler {
    fn compile_cmd(&self, unit: &CompileUnit, opts: &CompileOptions) -> Result<CannonicalCommand>;
    fn compile_batch_cmd(
        &self,
        units: &[CompileUnit],
        opts: &CompileOptions,
    ) -> Result<Vec<CannonicalCommand>> {
        let mut cmds = vec![];
        for unit in units {
            cmds.push(self.compile_cmd(unit, opts)?);
        }
        Ok(cmds)
    }
    fn get_dependencies(&self, file: &Path) -> Result<Vec<PathBuf>>;
    fn id(&self) -> &'static str;
    fn new() -> Self;
}

fn object_path(object_dir: &Path, source: &Path) -> Result<PathBuf> {
    let file_name = source
        .file_name()
        .context("Source does not have filename")?;
    Ok(object_dir.join(file_name).with_extension("o"))
}
