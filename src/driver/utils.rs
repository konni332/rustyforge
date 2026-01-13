use std::path::PathBuf;

use crate::{
    CoreResult, TargetKind,
    driver::{GlobalContext, runtime::Target},
};

impl<'ctx> GlobalContext<'ctx> {
    pub fn create_file_structure(&self, target: &Target) -> CoreResult<()> {
        std::fs::create_dir_all(self.get_object_dir(target))?;
        Ok(())
    }
    pub fn get_build_dir(&self) -> PathBuf {
        self.cwd.join("build")
    }
    pub fn get_profile_dir(&self) -> PathBuf {
        self.get_build_dir().join(&self.config.profile.name)
    }
    pub fn get_target_dir(&self, target: &Target) -> PathBuf {
        let base = self.get_profile_dir();
        base.join(match target.kind {
            TargetKind::Static => "static",
            TargetKind::Shared => "shared",
            TargetKind::Executable { .. } => "bin",
        })
    }
    pub fn get_object_dir(&self, target: &Target) -> PathBuf {
        self.get_target_dir(target).join("obj")
    }
    pub fn get_output_path(&self, target: &Target) -> PathBuf {
        let file_path = fmt::format_output_file(&target.kind, target.name);
        self.get_profile_dir().join(file_path)
    }
}

pub use fmt::format_output_file;
mod fmt {
    use std::path::PathBuf;

    use crate::{TargetKind, driver::runtime::Target};
    pub fn format_output_file(target_kind: &TargetKind, name: &str) -> PathBuf {
        match target_kind {
            TargetKind::Static => format_static(name),
            TargetKind::Shared => format_shared(name),
            TargetKind::Executable { .. } => format_executable(name),
        }
    }
    fn format_static(name: &str) -> PathBuf {
        #[cfg(windows)]
        return PathBuf::from(format!("{}.lib", name));
        #[cfg(target_os = "linux")]
        return PathBuf::from(format!("{}.a", name));
        #[cfg(target_os = "macos")]
        return PathBuf::from(format!("{}.a", name));
    }
    fn format_shared(name: &str) -> PathBuf {
        #[cfg(windows)]
        return PathBuf::from(format!("{}.dll", name));
        #[cfg(target_os = "linux")]
        return PathBuf::from(format!("{}.so", name));
        #[cfg(target_os = "macos")]
        return PathBuf::from(format!("{}.dylib", name));
    }
    fn format_executable(name: &str) -> PathBuf {
        #[cfg(windows)]
        return PathBuf::from(format!("{}.exe", name));
        #[cfg(unix)]
        return PathBuf::from(name);
    }
}
