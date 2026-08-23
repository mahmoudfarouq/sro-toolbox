//! Storage for the accounts subdomain.

pub mod memory;
pub mod sqlserver;

pub use memory::InMemoryAccountRepository;
pub use sqlserver::{SqlServerAccountRepository, SqlServerContext};
