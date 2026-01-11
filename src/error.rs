use std::{ops::Range, path::PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CoreError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid config error: {msg}")]
    InvalidConfig {
        path: PathBuf,
        src: String,
        span: Range<usize>,
        msg: String,
    },

    #[error("Toolchain error: {msg}")]
    ToolChain { msg: String, help: String },

    #[error("Cli validation error: {0}")]
    CliValidation(String),

    #[error("Unresolved profile name error: {0}")]
    UnresolvedProfileName(String),

    #[error("No eligible target specified error")]
    NoEligibleTarget,

    #[error("Manifest not found")]
    ManifestNotFound,

    #[error("Invalid manifest error: {msg}")]
    InvalidManifest {
        path: PathBuf,
        src: String,
        span: Range<usize>,
        msg: String,
    },

    #[error("Manifest validation error: {msg}")]
    ManifestValidation { msg: String, help: String },

    #[error("Toml serialization error: {0}")]
    TomlSerialization(#[from] toml::ser::Error),

    #[error("Fmt error: {0}")]
    Fmt(#[from] std::fmt::Error),

    #[error("Postcard serialization error: {0}")]
    PostcardSerialization(#[from] postcard::Error),

    #[error("Thread pool error: {0}")]
    ThreadPool(#[from] rayon::ThreadPoolBuildError),

    #[error("Ignore pattern error: {0}")]
    IgnorePattern(#[from] globset::Error),

    #[error("Failed to resolve dependencies for {path}: {err}")]
    ResolveDependency { path: PathBuf, err: String },
}

impl From<std::io::Error> for Box<CoreError> {
    fn from(e: std::io::Error) -> Self {
        Box::new(CoreError::Io(e))
    }
}

impl From<toml::ser::Error> for Box<CoreError> {
    fn from(e: toml::ser::Error) -> Self {
        Box::new(CoreError::TomlSerialization(e))
    }
}

impl From<std::fmt::Error> for Box<CoreError> {
    fn from(value: std::fmt::Error) -> Self {
        Box::new(value.into())
    }
}

impl From<postcard::Error> for Box<CoreError> {
    fn from(value: postcard::Error) -> Self {
        Box::new(value.into())
    }
}

impl From<rayon::ThreadPoolBuildError> for Box<CoreError> {
    fn from(value: rayon::ThreadPoolBuildError) -> Self {
        Box::new(value.into())
    }
}

impl From<globset::Error> for Box<CoreError> {
    fn from(value: globset::Error) -> Self {
        Box::new(value.into())
    }
}

pub type CoreResult<T> = Result<T, Box<CoreError>>;

#[macro_export]
macro_rules! internal_error {
    ($($arg:tt)*) => {
        use atty::Stream;
        let msg = format!("{}", format_args!($($arg)*));
        let version = env!("CARGO_PKG_VERSION");

        let url = format!(
            "https://github.com/konni332/rustyforge/issues/new?\
            template=bug_report.md&title=Internal+error&body=Version:+{}",
            version
        );
        let link = if atty::is(Stream::Stdout) {
            format!("\x1b]8;;{}\x07[Click here to report the bug]\x1b]8;;\x07", url)
        } else {
            url
        };
        let msg = format!("{}\nThis is a bug in RustyForge. Please report it. You can use the link below to create a new bug report issue.\n{}", msg, link);
        panic!("{}", msg);
    };
}

pub trait AnnotatedResult<T, E: std::error::Error> {
    fn unwrap_annotated(self, msg: &str) -> T;
}

impl<T, E: std::error::Error> AnnotatedResult<T, E> for Result<T, E> {
    fn unwrap_annotated(self, msg: &str) -> T {
        match self {
            Ok(value) => value,
            Err(_) => {
                let msg = format!("Called unwrap on a Result: {msg}");
                internal_error!("{}", msg);
            }
        }
    }
}
