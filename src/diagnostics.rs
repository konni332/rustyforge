#![allow(unused)]

use miette::{Diagnostic, NamedSource, SourceSpan};
use rustyforge_core::CoreError;
use thiserror::Error;

#[derive(Debug, Error, Diagnostic)]
pub enum RustyForgeReport {
    #[error(transparent)]
    #[diagnostic(transparent)]
    Diagnostic(RustyForgeDiagnostic),

    #[error(transparent)]
    Execption(CoreError),
}

#[derive(Debug, Error, Diagnostic)]
pub enum RustyForgeDiagnostic {
    #[error("Invalid command line arguments")]
    InvalidCliArgs {
        #[help]
        help: String,
    },

    #[error("Invalid manifest")]
    InvalidManifest {
        #[source_code]
        src: NamedSource<String>,

        #[label("{message}")]
        span: SourceSpan,

        message: String,
    },

    #[error("Invalid config")]
    InvalidConfig {
        #[source_code]
        src: NamedSource<String>,

        #[label("{message}")]
        span: SourceSpan,

        message: String,
    },

    #[error("No RustyForge.toml found")]
    ManifestNotFound {
        #[help]
        help: &'static str,
    },

    #[error("No eligable target found")]
    NoEligibleTarget {
        #[help]
        help: &'static str,
    },

    #[error("Unresolved profile name: {name}")]
    UnresolvedProfile {
        name: String,

        #[help]
        help: &'static str,
    },

    #[error("Toolchain error: {msg}")]
    Toolcahin {
        #[help]
        help: String,

        msg: String,
    },
}

impl From<CoreError> for RustyForgeReport {
    fn from(value: CoreError) -> Self {
        match RustyForgeDiagnostic::try_from(&value) {
            Ok(diagnostic) => RustyForgeReport::Diagnostic(diagnostic),
            Err(_) => RustyForgeReport::Execption(value),
        }
    }
}

impl From<Box<CoreError>> for RustyForgeReport {
    fn from(value: Box<CoreError>) -> Self {
        Self::from(*value)
    }
}

impl TryFrom<&CoreError> for RustyForgeDiagnostic {
    type Error = ();
    fn try_from(value: &CoreError) -> Result<Self, Self::Error> {
        match value {
            CoreError::InvalidManifest {
                path,
                src,
                span,
                msg,
            } => Ok(RustyForgeDiagnostic::InvalidManifest {
                src: NamedSource::new(path.display().to_string(), src.clone()),
                span: (span.start, span.len()).into(),
                message: msg.into(),
            }),
            CoreError::InvalidConfig {
                path,
                src,
                span,
                msg,
            } => Ok(RustyForgeDiagnostic::InvalidConfig {
                src: NamedSource::new(path.display().to_string(), src.clone()),
                span: (span.start, span.len()).into(),
                message: msg.into(),
            }),
            CoreError::CliValidation(help_msg) => Ok(RustyForgeDiagnostic::InvalidCliArgs {
                help: help_msg.to_string(),
            }),
            CoreError::UnresolvedProfileName(name) => Ok(RustyForgeDiagnostic::UnresolvedProfile {
                name: name.to_string(),
                help: "Make sure that the specified profile exists. Default profiles are: `dev`, `release` (custom profiles are not yet supported)",
            }),
            CoreError::NoEligibleTarget => Ok(RustyForgeDiagnostic::NoEligibleTarget {
                help: "Try adding binary or a library target to the RustyForge.toml",
            }),
            CoreError::ManifestNotFound => Ok(RustyForgeDiagnostic::ManifestNotFound {
                help: "Try creating a new project using `rustyforge new` or initializing one in this directory using `rustyforge init`",
            }),
            CoreError::ToolChain { msg, help } => Ok(RustyForgeDiagnostic::Toolcahin {
                help: msg.to_string(),
                msg: msg.to_string(),
            }),
            _ => Err(()),
        }
    }
}
