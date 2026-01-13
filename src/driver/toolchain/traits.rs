use std::path::{Path, PathBuf};

use crate::{
    CoreResult, TargetKind,
    driver::{
        cannonical_command::{CannonicalCommand, CannonicalCommandBuilder},
        runtime::{Profile, Target},
        toolchain::PROFILE_DEFINE_TEMPLATE,
    },
};

pub trait CCompiler: Send + Sync {
    fn new() -> Self
    where
        Self: Sized;
    fn compile_unit_cmd(
        &self,
        path: &Path,
        output: &Path,
        profile: &Profile,
        target: &Target,
        includes: &[PathBuf],
    ) -> CoreResult<CannonicalCommand>;
    fn get_dependencies(
        &self,
        src: &Path,
        profile: &Profile,
        target: &Target,
        includes: &[PathBuf],
    ) -> CoreResult<Vec<PathBuf>>;
}

pub trait CppCompiler: Send + Sync {
    fn new() -> Self
    where
        Self: Sized;
    fn compile_unit_cmd(
        &self,
        path: &Path,
        output: &Path,
        profile: &Profile,
        target: &Target,
        includes: &[PathBuf],
    ) -> CoreResult<CannonicalCommand>;
    fn get_dependencies(
        &self,
        src: &Path,
        profile: &Profile,
        target: &Target,
        includes: &[PathBuf],
    ) -> CoreResult<Vec<PathBuf>>;
}

pub trait Linker {
    fn new() -> Self
    where
        Self: Sized;
    fn link_objects(
        &self,
        target: &Target,
        profile: &Profile,
        srcs: &[PathBuf],
        contains_cpp: bool,
        lib_dirs: &[PathBuf],
        output: &Path,
    ) -> CoreResult<CannonicalCommand>;
}

pub trait Archiver {
    fn new() -> Self
    where
        Self: Sized;
    fn archiver_objects(&self, objs: &[PathBuf], output: PathBuf) -> CoreResult<CannonicalCommand>;
}
