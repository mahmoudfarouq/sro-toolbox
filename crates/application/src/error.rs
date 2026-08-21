//! Errors a use case can return.

use sro_toolbox_domain::DomainError;
use thiserror::Error;

/// Something a use case could not complete.
#[derive(Debug, Error)]
pub enum ApplicationError {
    /// The input violated a domain rule.
    #[error("invalid input: {0}")]
    Domain(#[from] DomainError),

    /// The thing being operated on does not exist.
    #[error("{kind} not found: {identifier}")]
    NotFound {
        /// What kind of thing was looked for.
        kind: &'static str,
        /// How it was identified.
        identifier: String,
    },

    /// A port failed.
    #[error("{0}")]
    Port(#[from] PortError),
}

/// Result type for use cases.
pub type ApplicationResult<T> = Result<T, ApplicationError>;

/// A failure originating in an adapter rather than in the domain.
///
/// Adapters map their own concerns — connection failures, malformed rows, a
/// missing PK2 file — onto this, so use cases never depend on a particular
/// storage technology.
#[derive(Debug, Error)]
pub enum PortError {
    /// The backing store could not be reached or read.
    #[error("storage failure: {0}")]
    Storage(String),

    /// The adapter has not been built yet.
    ///
    /// Present so the command surface can be complete and navigable while the
    /// adapters behind it are still stubs.
    #[error("not implemented yet: {0}")]
    Unimplemented(&'static str),
}

/// Result type for ports.
pub type PortResult<T> = Result<T, PortError>;
