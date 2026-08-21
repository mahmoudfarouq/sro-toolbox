//! Use cases, and the ports they depend on.
//!
//! This layer holds *what the tool does*, expressed without reference to how it
//! is invoked or where the data lives. A use case takes a plain input struct and
//! returns a plain output struct, so the same one serves a CLI command, an HTTP
//! handler, or a test.
//!
//! Dependencies point inwards only: this crate depends on the domain and on the
//! traits in [`ports`], never on an adapter. See `docs/architecture.md`.

pub mod error;
pub mod ports;
pub mod use_cases;

pub use error::{ApplicationError, ApplicationResult};
