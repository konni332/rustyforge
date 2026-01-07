use std::path::{Path, PathBuf};

use crate::config::project::LinkTargetKind;

pub struct LinkUnit<'a> {
    pub kind: LinkTargetKind,
    pub objects: &'a [PathBuf],
    pub output: &'a Path,
    pub user_flags: &'a [String],

    pub soname: &'a Option<String>,

    pub lib_dirs: Vec<PathBuf>,
    pub libs: Vec<String>,
}

pub struct LinkOptions {
    pub target: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct LinkingResult {
    pub exe_path: Option<PathBuf>,
    pub static_lib: Option<PathBuf>,
    pub shared_lid: Option<PathBuf>,
}
