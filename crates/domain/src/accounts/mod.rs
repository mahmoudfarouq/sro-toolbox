//! The accounts subdomain: login identities, their privileges and their bans.
//!
//! Layout, which every subdomain follows:
//!
//! - [`account`] — the entities and value objects.
//! - [`repository`] — the port persistence must satisfy.
//! - [`ban_account`] — one use case, one file.

pub mod account;
pub mod ban_account;
pub mod repository;

pub use account::{Account, AccountId, AccountName, AccountStatus, BanDuration, BanReason};
pub use ban_account::{BanAccount, BanAccountInput, BanAccountOutput};
pub use repository::AccountRepository;
