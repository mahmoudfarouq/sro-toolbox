//! The business logic and the entities it operates on, split by subdomain.
//!
//! Each subdomain is a module holding everything about one area:
//!
//! ```text
//! accounts/
//!   account.rs        the entities and value objects
//!   repository.rs     the port: what persistence must provide
//!   ban_account.rs    one use case
//!   mod.rs            re-exports
//! ```
//!
//! One file per use case. A new operation is a new file, not a new arm in a
//! growing match.
//!
//! ## Why the ports live here
//!
//! The repository traits are declared alongside the logic that needs them and
//! implemented in `sro-toolbox-storage`. That inverts what would otherwise be a
//! dependency on a database driver: this crate names what it needs, and
//! something else supplies it. Nothing here knows SQL Server exists.
//!
//! The cost is that this crate is not dependency-free — declaring an async port
//! means depending on `async-trait`. That is the deliberate trade for having no
//! separate application layer; see `docs/adr/0005-domain-owned-use-cases.md`.

pub mod accounts;
pub mod audit;
pub mod error;

pub use error::{
    DomainError, DomainResult, RepositoryError, RepositoryResult, UseCaseError, UseCaseResult,
};
