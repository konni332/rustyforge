#![allow(unused)]

pub const RUSTYFORGE_VERSION: &str = "0.4.1";

pub static SHELL: OnceLock<Arc<Mutex<Shell>>> = OnceLock::new();

mod cli;
mod config;
mod diagnostics;
mod driver;
mod error;
pub mod shell;
mod utils;

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::OnceLock;

pub use cli::Cli;
pub use cli::CliCommand;
pub use config::ToolConfig;
pub use config::ToolchainExecutable;
pub use config::manifest;
pub use diagnostics::RustyForgeReport;
pub use driver::BuildResult;
pub use driver::GlobalContext;
pub use driver::ProjectInfo;
pub use driver::RunTimeConfig;
pub use driver::TargetKind;
pub use error::AnnotatedResult;
pub use error::CoreError;
pub use error::CoreResult;
pub use shell::ui::ProgressBar;

use crate::shell::Shell;

fn shell() -> &'static Arc<Mutex<Shell>> {
    SHELL.get_or_init(|| Arc::new(Mutex::new(Shell::new())))
}

pub fn with_shell<F, R>(f: F) -> R
where
    F: FnOnce(&mut Shell) -> R,
{
    let shell = shell();
    let mut guard = shell.lock().unwrap_annotated("shell poisoned");
    f(&mut guard)
}
