//! Errors raised when a domain invariant is violated.

use thiserror::Error;

/// A value rejected by the domain because it would break an invariant.
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
        /// The field that would overflow.
        field: &'static str,
    },
}

/// Result type for domain operations.
pub type DomainResult<T> = Result<T, DomainError>;
