//! In-process adapters.
//!
//! Useful beyond tests: running the CLI against these gives a safe sandbox for
//! trying a command out, and they are the reference for what a real adapter has
//! to do.

pub mod account;
pub mod audit;

pub use account::InMemoryAccountRepository;
pub use audit::InMemoryAuditLog;
