use std::path::Path;

use crate::{
    CoreResult,
    driver::{
        cannonical_command::CannonicalCommand,
        runtime::{Profile, Target},
    },
};

pub trait CCompiler: Send + Sync {
    fn id(&self) -> &'static str;
    fn compile_unit_cmd(
        &self,
        path: &Path,
        output: &Path,
        profile: &Profile,
        target: &Target,
    ) -> CoreResult<CannonicalCommand>;
}

pub trait CppCompiler: Send + Sync {
    fn id(&self) -> &'static str;
    fn compile_unit_cmd(
        &self,
        path: &Path,
        output: &Path,
        profile: &Profile,
        target: &Target,
    ) -> CoreResult<CannonicalCommand>;
}

pub trait Linker {
    fn id(&self) -> &'static str;
}

pub trait Archiver {
    fn id(&self) -> &'static str;
}
