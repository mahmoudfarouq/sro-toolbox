//! Storage adapters: concrete implementations of the ports the domain declares.
//!
//! Split by subdomain, mirroring `sro-toolbox-domain`, so the adapter for a
//! subdomain sits next to nothing else:
//!
//! ```text
//! accounts/
//!   memory.rs      in-process, for tests and a safe sandbox
//!   sqlserver.rs   the real target, currently stubs
//! audit/
//!   memory.rs
//! ```
//!
//! Nothing in the domain references this crate. The composition root in the
//! interface crate chooses which adapter to use, which is what allows the tool
//! to run against fakes, a real database, or a mixture.

pub mod accounts;
pub mod audit;
