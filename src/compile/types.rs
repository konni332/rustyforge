use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone)]
pub struct CompileUnit<'a> {
    pub source: &'a Path,
    pub includes: &'a [PathBuf],
    pub defines: &'a [String],
}

#[derive(Debug, Clone)]
pub struct CompileOptions {
    pub profile: Profile,
    pub object_dir: PathBuf,
    pub user_flags: Vec<String>,
    pub defines: Vec<String>,
    pub target: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Profile {
    Release,
    Debug,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CannonicalCommand {
    program: String,
    args: Vec<String>,
    envs: Vec<(String, String)>,
    cwd: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CannonicalCommandBuilder {
    program: String,
    args: Vec<String>,
    envs: Vec<(String, String)>,
    cwd: Option<PathBuf>,
}

impl CannonicalCommandBuilder {
    pub fn new<S: Into<String>>(program: S) -> Self {
        Self {
            program: program.into(),
            args: vec![],
            envs: vec![],
            cwd: None,
        }
    }
    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        let arg = arg.as_ref().to_string_lossy().to_string();
        self.args.push(arg);
        self
    }
    pub fn args<S: AsRef<OsStr>, I: IntoIterator<Item = S>>(&mut self, args: I) -> &mut Self {
        for arg in args {
            self.arg(arg);
        }
        self
    }
    pub fn finish(self) -> CannonicalCommand {
        let program = self.program;
        let args = self.args;
        let mut envs = self.envs;
        envs.sort();
        let cwd = self.cwd;
        CannonicalCommand {
            program,
            args,
            envs,
            cwd,
        }
    }
}

impl From<&CannonicalCommand> for std::process::Command {
    fn from(value: &CannonicalCommand) -> Self {
        let mut cmd = std::process::Command::new(&value.program);
        cmd.args(&value.args);
        for env in value.envs.iter() {
            cmd.env(&env.0, &env.1);
        }
        if let Some(cwd) = &value.cwd {
            cmd.current_dir(cwd);
        }
        cmd
    }
}
