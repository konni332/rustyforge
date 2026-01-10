use std::{path::PathBuf, sync::Arc};

use globset::GlobSet;

use crate::driver::GlobalContext;

impl<'ctx> GlobalContext<'ctx> {
    fn discover_c_files(&self, ignore: Arc<GlobSet>) -> Vec<PathBuf> {
        let walkdir = self.create_entry_iter_ignore_subpackage(ignore);
        let mut c_files: Vec<PathBuf> = walkdir
            .filter_map(|res| res.ok())
            .filter_map(|entry| {
                if entry.path().is_file() && entry.path().extension().is_some_and(|ext| ext == "c")
                {
                    Some(entry.path().to_path_buf())
                } else {
                    None
                }
            })
            .collect();
        c_files.dedup();
        c_files.sort();
        c_files
    }
    fn discover_include_dirs(&self, ignore: Arc<GlobSet>) -> Vec<PathBuf> {
        let walkdir = self.create_entry_iter_ignore_subpackage(ignore);
        let mut dirs: Vec<PathBuf> = walkdir
            .filter_map(|res| res.ok())
            .filter_map(|entry| {
                if entry.path().is_file() && entry.path().extension().is_some_and(|ext| ext == "h")
                {
                    Some(entry.parent_path().to_path_buf())
                } else {
                    None
                }
            })
            .collect();
        dirs.dedup();
        sort_include_dirs(&mut dirs);
        dirs
    }
    fn create_entry_iter_ignore_subpackage(
        &self,
        ignore: Arc<GlobSet>,
    ) -> jwalk::DirEntryIter<((), ())> {
        jwalk::WalkDir::new(&self.cwd)
            .parallelism(jwalk::Parallelism::RayonExistingPool {
                pool: self.pool.clone(),
                busy_timeout: None,
            })
            .process_read_dir(move |depth, dir_path, _state, children| {
                if ignore.is_match(dir_path) {
                    children.clear();
                    return;
                }
                if depth.unwrap_or(0) > 0
                    && children.iter().any(|res| {
                        res.as_ref().is_ok_and(|entry| {
                            entry
                                .path()
                                .file_name()
                                .is_some_and(|n| n == "RustyForge.toml")
                        })
                    })
                {
                    children.clear();
                    return;
                }
                children.retain(|res| {
                    res.as_ref()
                        .is_ok_and(|child| !ignore.is_match(child.path()))
                });
            })
            .into_iter()
    }
}

fn sort_include_dirs(dirs: &mut [PathBuf]) {
    dirs.sort_by(|a, b| {
        let len_cmp = a.components().count().cmp(&b.components().count());

        if len_cmp != std::cmp::Ordering::Equal {
            len_cmp
        } else {
            a.as_os_str().cmp(b.as_os_str())
        }
    });
}
