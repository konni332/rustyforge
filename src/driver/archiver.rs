use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use crate::config::tool_config::ArchiverKind;
use crate::{
    CoreResult,
    driver::{
        cannonical_command::{CannonicalCommand, CannonicalCommandBuilder},
        toolchain::Archiver,
    },
};

pub struct ArchiverContext<'ctx> {
    output: &'ctx Path,
    seed: u64,
    objs: &'ctx [PathBuf],
    archiver: ArchiverKind,
}

impl<'ctx> ArchiverContext<'ctx> {
    pub fn new(
        output: &'ctx Path,
        seed: u64,
        objs: &'ctx [PathBuf],
        archiver: ArchiverKind,
    ) -> Self {
        Self {
            output,
            seed,
            objs,
            archiver,
        }
    }
    pub fn build(&self) -> CoreResult<(CannonicalCommand, u64)> {
        match self.archiver {
            ArchiverKind::Lib => self.build_archiver_cmd::<MsvcArchiver>(),
            ArchiverKind::Ar => self.build_archiver_cmd::<ArArchiver>(),
            ArchiverKind::LlvmAr => self.build_archiver_cmd::<LlvmArchiver>(),
        }
    }
    fn build_archiver_cmd<A: Archiver>(&self) -> CoreResult<(CannonicalCommand, u64)> {
        let archiver = A::new();
        let cmd = archiver.archiver_objects(self.objs, self.output)?;

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

pub struct ArArchiver;

impl Archiver for ArArchiver {
    fn new() -> Self
    where
        Self: Sized,
    {
        ArArchiver
    }
    fn archiver_objects(
        &self,
        objs: &[std::path::PathBuf],
        output: &Path,
    ) -> crate::CoreResult<super::cannonical_command::CannonicalCommand> {
        let mut cmd = CannonicalCommandBuilder::new("ar");
        cmd.arg("rcs").arg(output).args(objs);
        Ok(cmd.finish())
    }
}

pub struct LlvmArchiver;

impl Archiver for LlvmArchiver {
    fn new() -> Self
    where
        Self: Sized,
    {
        LlvmArchiver
    }
    fn archiver_objects(
        &self,
        objs: &[std::path::PathBuf],
        output: &Path,
    ) -> crate::CoreResult<super::cannonical_command::CannonicalCommand> {
        let mut cmd = CannonicalCommandBuilder::new("llvm-ar");
        cmd.arg("rcs").arg(output).args(objs);
        Ok(cmd.finish())
    }
}

pub struct MsvcArchiver;

impl Archiver for MsvcArchiver {
    fn new() -> Self
    where
        Self: Sized,
    {
        MsvcArchiver
    }
    fn archiver_objects(
        &self,
        objs: &[std::path::PathBuf],
        output: &Path,
    ) -> crate::CoreResult<super::cannonical_command::CannonicalCommand> {
        let mut cmd = CannonicalCommandBuilder::new("lib.exe");
        cmd.arg(format!("/OUT:{}", output.display())).args(objs);
        Ok(cmd.finish())
    }
}
