//! Storage adapters: concrete implementations of the ports the domain declares.
//!
//! Organised adapter first, then subdomain, then one file per operation:
//!
//! ```text
//! sqlserver/account/find_by_name.rs
//! sqlserver/account/save.rs
//! memory/account/find_by_name.rs
//! memory/account/save.rs
//! ```
//!
//! A port's `impl` block lives in the subdomain's `mod.rs` and delegates to
//! those files, so each operation — its SQL, its mapping, its tests — is one
//! self-contained unit.
//!
//! Two adapters:
//!
//! - [`memory`] — in-process. Used by tests, by the CLI's sandbox, and as the
//!   reference for what a real adapter has to do.
//! - [`sqlserver`] — the real target, talking to `SRO_VT_ACCOUNT` over TDS.
//!
//! Nothing in the domain references this crate. The composition root in the
//! interface crate chooses which adapter to use.

pub mod memory;
pub mod sqlserver;
