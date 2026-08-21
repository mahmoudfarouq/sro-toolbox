//! The domain model: what a Silkroad server *is*, independent of how it is
//! stored or operated.
//!
//! This crate has no I/O, no database, no CLI and no async. It holds types that
//! enforce their own invariants, so an invalid value cannot be constructed. That
//! is what lets the layers above trust their inputs.
//!
//! Only [`account`] exists so far. Characters, items, currencies and world
//! geometry belong here too and will be added as the use cases that need them
//! are written.
//!
//! See `docs/architecture.md` for how this fits the rest of the workspace.

pub mod account;
pub mod error;

pub use error::{DomainError, DomainResult};
