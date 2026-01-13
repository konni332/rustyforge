use crate::driver::{cannonical_command::CannonicalCommandBuilder, toolchain::Archiver};

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
        output: std::path::PathBuf,
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
        output: std::path::PathBuf,
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
        output: std::path::PathBuf,
    ) -> crate::CoreResult<super::cannonical_command::CannonicalCommand> {
        let mut cmd = CannonicalCommandBuilder::new("lib.exe");
        cmd.arg(format!("/OUT:{}", output.display())).args(objs);
        Ok(cmd.finish())
    }
}
