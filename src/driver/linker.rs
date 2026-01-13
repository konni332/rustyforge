use crate::{
    CoreError, CoreResult,
    config::tool_config::LinkerKind,
    driver::{
        cannonical_command::CannonicalCommand,
        runtime::{Profile, Target},
        toolchain::{Clang, Gcc, Linker, Msvc},
    },
    internal_error,
};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

pub struct LinkContext<'ctx> {
    lib_dirs: &'ctx [PathBuf],
    objs: &'ctx [PathBuf],
    contains_cpp: bool,
    target: &'ctx Target<'ctx>,
    profile: &'ctx Profile,
    linker: LinkerKind,
    seed: u64,
}

impl<'ctx> LinkContext<'ctx> {
    pub fn new(
        objs: &'ctx [PathBuf],
        contains_cpp: bool,
        lib_dirs: &'ctx [PathBuf],
        target: &'ctx Target,
        profile: &'ctx Profile,
        linker: LinkerKind,
        seed: u64,
    ) -> Self {
        Self {
            lib_dirs,
            objs,
            contains_cpp,
            target,
            profile,
            linker,
            seed,
        }
    }
    pub fn build(&self, output: &Path) -> CoreResult<(CannonicalCommand, u64)> {
        match self.linker {
            LinkerKind::Gcc => self.build_linker_cmd::<Gcc>(output),
            LinkerKind::Clang => self.build_linker_cmd::<Clang>(output),
            LinkerKind::Msvc => self.build_linker_cmd::<Msvc>(output),
            LinkerKind::Lld => Err(Box::new(CoreError::NotSupported {
                msg: "lld linker".into(),
                note: r#"
Support is planned for the near future, which is why the config already accepts it.
For Now please use an alternative like: clang(ld internally), gcc(ld internally), msvc
                "#
                .into(),
            })),
        }
    }
    fn build_linker_cmd<L: Linker + Send + Sync>(
        &self,
        output: &Path,
    ) -> CoreResult<(CannonicalCommand, u64)> {
        let linker = L::new();
        let cmd = linker.link_objects(
            self.target,
            self.profile,
            self.objs,
            self.contains_cpp,
            self.lib_dirs,
            output,
        )?;

        let hash = self.get_command_hash(&cmd)?;

        Ok((cmd, hash))
    }
    fn get_command_hash(&self, cmd: &CannonicalCommand) -> CoreResult<u64> {
        let mut hasher = twox_hash::XxHash64::with_seed(self.seed);
        cmd.hash(&mut hasher);
        for obj in self.objs {
            let content = std::fs::read(obj)?;
            content.hash(&mut hasher);
        }

        Ok(hasher.finish())
    }
}
