//! CLI-level errors and the exit codes they map to.

use std::process::ExitCode;

use sro_toolbox_domain::{RepositoryError, UseCaseError};
use thiserror::Error;

/// Something the CLI could not do.
#[derive(Debug, Error)]
pub(crate) enum CliError {
    /// A use case failed.
    #[error(transparent)]
    UseCase(#[from] UseCaseError),

    /// The command exists but is not wired up yet.
    #[error("`{command}` is not implemented yet")]
    NotImplemented {
        /// The command that was invoked.
        command: &'static str,
    },

    /// Results could not be rendered.
    #[error("could not render output: {0}")]
    Render(#[from] serde_json::Error),
}

impl CliError {
    /// The exit code this error should produce.
    ///
    /// Distinct codes let scripts branch on the kind of failure instead of
    /// parsing messages.
    #[must_use]
    pub(crate) fn exit_code(&self) -> ExitCode {
        match self {
            // Bad input from the operator.
            Self::UseCase(UseCaseError::Domain(_)) => ExitCode::from(2),
            // The target does not exist.
            Self::UseCase(UseCaseError::NotFound { .. }) => ExitCode::from(3),
            // The adapter is a stub.
            Self::UseCase(UseCaseError::Repository(RepositoryError::Unimplemented(_)))
            | Self::NotImplemented { .. } => ExitCode::from(4),
            // Everything else.
            Self::UseCase(UseCaseError::Repository(_)) | Self::Render(_) => ExitCode::from(1),
        }
    }
}
