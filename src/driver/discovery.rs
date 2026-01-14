use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use globset::GlobSet;

use crate::driver::{GlobalContext, runtime::Target};

impl<'ctx> GlobalContext<'ctx> {
    pub fn discover_obj_files(&self, target: &Target) -> Vec<PathBuf> {
        #[cfg(windows)]
        let obj_ext = "obj";
        #[cfg(not(windows))]
        let obj_ext = "o";

        let objs_dir = self.get_object_dir(target);
        let walkdir =
            self.create_entry_iter_ignore_subpackage(&objs_dir, Arc::new(GlobSet::empty()));
        walkdir
            .filter_map(|res| res.ok())
            .filter_map(|entry| {
                if entry.path().is_file()
                    && entry.path().extension().is_some_and(|ext| ext == obj_ext)
                {
                    Some(entry.path().to_path_buf())
                } else {
                    None
                }
            })
            .collect()
    }
    pub fn discover_c_files(&self, ignore: Arc<GlobSet>) -> Vec<PathBuf> {
        let walkdir = self.create_entry_iter_ignore_subpackage(&self.cwd, ignore);
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
    pub fn discover_cpp_files(&self, ignore: Arc<GlobSet>) -> Vec<PathBuf> {
        let walkdir = self.create_entry_iter_ignore_subpackage(&self.cwd, ignore);
        let mut cpp_files: Vec<PathBuf> = walkdir
            .filter_map(|res| res.ok())
            .filter_map(|entry| {
                if entry.path().is_file()
                    && entry
                        .path()
                        .extension()
                        .is_some_and(|ext| ext == "cpp" || ext == "cxx" || ext == "cc")
                {
                    Some(entry.path().to_path_buf())
                } else {
                    None
                }
            })
            .collect();
        cpp_files.dedup();
        cpp_files.sort();
        cpp_files
    }
    pub fn discover_include_dirs(&self, ignore: Arc<GlobSet>) -> Vec<PathBuf> {
        let walkdir = self.create_entry_iter_ignore_subpackage(&self.cwd, ignore);

        let mut dirs = Vec::<PathBuf>::new();

        for entry in walkdir.filter_map(Result::ok) {
            let path = entry.path();

            if path.is_file()
                && path
                    .extension()
                    .is_some_and(|ext| (ext == "h") || (ext == "hpp"))
            {
                let mut cur = path.parent();

                while let Some(dir) = cur {
                    // Stop BEFORE walk root
                    if dir == self.cwd {
                        break;
                    }

                    dirs.push(dir.to_path_buf());
                    cur = dir.parent();
                }
            }
        }
        sort_include_dirs(&mut dirs);
        dirs.dedup();
        dirs
    }
    fn create_entry_iter_ignore_subpackage(
        &self,
        root: &Path,
        ignore: Arc<GlobSet>,
    ) -> jwalk::DirEntryIter<((), ())> {
        let root = root.canonicalize().unwrap_or(root.to_path_buf());
        jwalk::WalkDir::new(&root)
            .parallelism(jwalk::Parallelism::RayonExistingPool {
                pool: self.pool.clone(),
                busy_timeout: None,
            })
            .process_read_dir(move |depth, dir_path, _state, children| {
                if ignore.is_match(to_relative(&root, dir_path)) {
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
                        .is_ok_and(|child| !ignore.is_match(to_relative(&root, &child.path())))
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

fn to_relative<'a>(cwd: &Path, path: &'a Path) -> &'a Path {
    path.strip_prefix(cwd).unwrap_or(path)
}
