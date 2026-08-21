//! In-process adapters.
//!
//! Useful beyond tests: running the CLI against these gives a safe sandbox for
//! trying a command out, and they are the reference for what a real adapter has
//! to do.

mod account_repository;
mod audit_log;

pub use account_repository::InMemoryAccountRepository;
pub use audit_log::InMemoryAuditLog;
