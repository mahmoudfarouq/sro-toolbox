//! Adapters for the server's own SQL Server databases.
//!
//! Stubs. Each reports [`PortError::Unimplemented`] rather than panicking, so
//! the command surface can be explored and tested before any query exists, and
//! so a half-built adapter can never silently appear to work.
//!
//! The real implementations will target `SRO_VT_ACCOUNT`, `SRO_VT_SHARD`,
//! `SRO_VT_LOG` and `SRO_VT_SHARDLOG` over a connection pool held by
//! [`SqlServerContext`].

use async_trait::async_trait;
use sro_toolbox_application::error::{PortError, PortResult};
use sro_toolbox_application::ports::AccountRepository;
use sro_toolbox_domain::account::{Account, AccountName};

/// Shared connection state for the SQL Server adapters.
///
/// A placeholder for the pool. It carries the connection string so the shape of
/// configuration is settled even though nothing connects yet.
#[derive(Debug, Clone)]
pub struct SqlServerContext {
    connection_string: String,
}

impl SqlServerContext {
    /// Describe how to reach the databases.
    #[must_use]
    pub fn new(connection_string: impl Into<String>) -> Self {
        Self {
            connection_string: connection_string.into(),
        }
    }

    /// The configured connection string.
    #[must_use]
    pub fn connection_string(&self) -> &str {
        &self.connection_string
    }
}

/// Accounts read from `SRO_VT_ACCOUNT`.
#[derive(Debug, Clone)]
pub struct SqlServerAccountRepository {
    #[allow(
        dead_code,
        reason = "held for the queries this adapter will issue once implemented"
    )]
    context: SqlServerContext,
}

impl SqlServerAccountRepository {
    /// Build the adapter over a connection context.
    #[must_use]
    pub const fn new(context: SqlServerContext) -> Self {
        Self { context }
    }
}

#[async_trait]
impl AccountRepository for SqlServerAccountRepository {
    async fn find_by_name(&self, _name: &AccountName) -> PortResult<Option<Account>> {
        Err(PortError::Unimplemented(
            "reading accounts from SRO_VT_ACCOUNT",
        ))
    }

    async fn save(&self, _account: &Account) -> PortResult<()> {
        Err(PortError::Unimplemented(
            "writing accounts to SRO_VT_ACCOUNT",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn the_stub_reports_unimplemented_rather_than_pretending() {
        let repository =
            SqlServerAccountRepository::new(SqlServerContext::new("Server=localhost;"));
        let name = AccountName::parse("player01").expect("valid");

        let error = repository
            .find_by_name(&name)
            .await
            .expect_err("stub should fail");

        assert!(matches!(error, PortError::Unimplemented(_)));
    }
}
