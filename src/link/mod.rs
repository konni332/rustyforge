use anyhow::Result;

use crate::{
    compile::types::CannonicalCommand,
    link::types::{LinkOptions, LinkUnit},
};

mod clang;
mod driver;
mod gcc;
mod msvc;
mod types;

pub use clang::ClangLinker;
pub use driver::LinkerDirver;
pub use gcc::GccLinker;
pub use msvc::MsvcLinker;
pub use types::LinkingResult;

pub trait Linker {
    fn new() -> Self;
    fn link_cmd(&self, unit: &LinkUnit, opts: &LinkOptions) -> Result<CannonicalCommand>;
}
