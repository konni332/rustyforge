pub const RUSTYFORGE_VERSION: &str = "0.4.1";

mod cli;
mod compiler;
mod config;
mod driver;
mod error;
mod utils;

pub use cli::Cli;
pub use cli::CliCommand;
pub use config::ToolConfig;
pub use config::ToolchainExecutable;
pub use config::manifest;
pub use driver::RunTimeConfig;
pub use driver::TargetKind;
pub use error::AnnotatedResult;
pub use error::CoreError;
pub use error::CoreResult;

pub use driver::get_project_info_string;
