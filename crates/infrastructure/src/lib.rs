//! Adapters: concrete implementations of the ports the application layer
//! declares.
//!
//! Nothing here is referenced by the application layer. The composition root in
//! the interface crate picks which adapter to use, which is what allows the
//! whole tool to run against in-memory fakes, a real database, or a mixture.
//!
//! Two families exist today:
//!
//! - [`memory`] — everything held in process. Used by tests, by `--dry-run`,
//!   and to demonstrate the wiring end to end.
//! - [`sqlserver`] — the real target. Currently stubs that report
//!   [`sro_toolbox_application::error::PortError::Unimplemented`], so the
//!   command surface is navigable before the queries exist.

pub mod memory;
pub mod sqlserver;
