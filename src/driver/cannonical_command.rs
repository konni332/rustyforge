use std::{ffi::OsStr, hash::Hash, path::PathBuf};

pub struct CannonicalCommandBuilder {
    program: String,
    args: Vec<String>,
    env_args: Vec<(String, String)>,
    cwd: Option<PathBuf>,
}

impl CannonicalCommandBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            program: name.to_string(),
            args: vec![],
            env_args: vec![],
            cwd: None,
        }
    }
    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut CannonicalCommandBuilder {
        let arg = arg.as_ref().to_string_lossy().to_string();
        self.args.push(arg);
        self
    }
    pub fn args<S: AsRef<OsStr>>(&mut self, args: &[S]) -> &mut CannonicalCommandBuilder {
        for arg in args {
            self.arg(arg);
        }
        self
    }
    pub fn env<S: AsRef<OsStr>>(&mut self, env: (S, S)) {
        self.env_args.push((
            env.0.as_ref().to_string_lossy().to_string(),
            env.1.as_ref().to_string_lossy().to_string(),
        ));
    }
    pub fn current_dir(&mut self, path: PathBuf) -> &mut CannonicalCommandBuilder {
        self.cwd = Some(path);
        self
    }
    pub fn finish(self) -> CannonicalCommand {
        let program = self.program;
        let args = self.args;
        let mut env_args = self.env_args;
        env_args.sort();
        CannonicalCommand {
            program,
            args,
            env_args,
            cwd: self.cwd,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CannonicalCommand {
    program: String,
    args: Vec<String>,
    env_args: Vec<(String, String)>,
    cwd: Option<PathBuf>,
}

impl Hash for CannonicalCommand {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.program.hash(state);

        let mut env_args = self.env_args.clone();
        env_args.sort();
        env_args.hash(state);

        self.args.hash(state);
        self.cwd.hash(state);
    }
}

impl From<&CannonicalCommand> for std::process::Command {
    fn from(value: &CannonicalCommand) -> Self {
        let mut cmd = std::process::Command::new(&value.program);
        cmd.args(&value.args);
        if let Some(cwd) = &value.cwd {
            cmd.current_dir(cwd);
        }
        cmd
    }
}
