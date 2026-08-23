//! Three kinds of failure, kept apart because callers treat them differently.
//!
//! - [`DomainError`] — a value was rejected by a rule. The operator's mistake.
//! - [`RepositoryError`] — a port failed. Operational.
//! - [`UseCaseError`] — what a use case returns; wraps both and adds
//!   "the thing does not exist", which is neither of the above.

use thiserror::Error;

/// A value rejected because it would break an invariant.
///
/// These describe *why* a value is unacceptable, never where it came from, so
/// the same error is meaningful whether the input arrived from a CLI argument,
/// an HTTP body, or a database row.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    /// A required text value was empty or only whitespace.
    #[error("{field} must not be empty")]
    Empty {
        /// The field that was empty.
        field: &'static str,
    },

    /// A text value exceeded the length the game format allows.
    #[error("{field} must be at most {max} characters, got {actual}")]
    TooLong {
        /// The field that was too long.
        field: &'static str,
        /// The maximum permitted length.
        max: usize,
        /// The length supplied.
        actual: usize,
    },

    /// A text value contained characters the game does not accept.
    #[error("{field} contains an unsupported character: {character:?}")]
    InvalidCharacter {
        /// The field that was rejected.
        field: &'static str,
        /// The first offending character.
        character: char,
    },

    /// A numeric value fell outside its permitted range.
    #[error("{field} must be between {min} and {max}, got {actual}")]
    OutOfRange {
        /// The field that was out of range.
        field: &'static str,
        /// Lowest permitted value.
        min: i64,
        /// Highest permitted value.
        max: i64,
        /// The value supplied.
        actual: i64,
    },

    /// An arithmetic operation on a currency or counter left its valid range.
    #[error("{operation} would take {field} out of its valid range")]
    Overflow {
        /// The operation attempted.
        operation: &'static str,
        /// The field affected.
        field: &'static str,
    },
}

/// Result of constructing or mutating a domain value.
pub type DomainResult<T> = Result<T, DomainError>;

/// A failure originating in a port implementation rather than in the logic.
///
/// Adapters map their own concerns — a dropped connection, a malformed row, a
/// missing PK2 file — onto this, so the business logic never depends on a
/// particular storage technology.
#[derive(Debug, Error)]
pub enum RepositoryError {
    /// The backing store could not be reached or read.
    #[error("storage failure: {0}")]
    Storage(String),

    /// The adapter has not been built yet.
    ///
    /// Present so the command surface can be complete and navigable while the
    /// adapters behind it are still stubs. An error rather than a panic, so a
    /// stub can never be mistaken for working code.
    #[error("not implemented yet: {0}")]
    Unimplemented(&'static str),
}

/// Result of a port call.
pub type RepositoryResult<T> = Result<T, RepositoryError>;

/// What a use case returns.
#[derive(Debug, Error)]
pub enum UseCaseError {
    /// The input violated a rule.
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
    #[error(transparent)]
    Repository(#[from] RepositoryError),
}

/// Result of executing a use case.
pub type UseCaseResult<T> = Result<T, UseCaseError>;
