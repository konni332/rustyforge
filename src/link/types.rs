use std::path::PathBuf;

use crate::config::project::LinkTargetKind;

pub struct LinkUnit<'a> {
    pub kind: LinkTargetKind,
    pub objects: &'a [PathBuf],
    pub output: PathBuf,
    pub user_flags: &'a [String],

    pub lib_dirs: Vec<PathBuf>,
    pub libs: Vec<String>,
}

pub struct LinkOptions {
    pub target: Option<String>,
}
