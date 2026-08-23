//! The composition root.
//!
//! The single place that decides which adapter satisfies which port. Nothing
//! else in the CLI constructs an adapter, so pointing the tool at a real
//! database — or introducing a web interface that reuses the same use cases — is
//! a change here and nowhere else.

use std::sync::Arc;

use sro_toolbox_domain::accounts::{
    Account, AccountId, AccountName, AccountRepository, AccountStatus, BanAccount,
};
use sro_toolbox_domain::audit::AuditLog;
use sro_toolbox_storage::memory::{InMemoryAccountRepository, InMemoryAuditLog};
use sro_toolbox_storage::sqlserver::{SqlServerAccountRepository, SqlServerContext};

use crate::error::CliError;

/// Everything the commands need, already wired.
pub(crate) struct Context {
    accounts: Arc<dyn AccountRepository>,
    audit: Arc<dyn AuditLog>,
}

impl Context {
    /// Choose adapters from the options given.
    ///
    /// With a connection string, the tool talks to a real server. Without one it
    /// runs against an in-process sandbox, so a command can be tried out without
    /// a database and without the risk of touching live player data.
    ///
    /// # Errors
    ///
    /// Returns [`CliError`] if the connection string cannot be parsed.
    pub(crate) fn from_options(database: Option<&str>) -> Result<Self, CliError> {
        match database {
            Some(connection_string) => Self::sql_server(connection_string),
            None => Ok(Self::in_memory()),
        }
    }

    /// Adapters backed by the server's SQL Server databases.
    ///
    /// The connection string is only parsed here; nothing connects until a
    /// command actually needs to.
    ///
    /// # Errors
    ///
    /// Returns [`CliError`] if the connection string cannot be parsed.
    pub(crate) fn sql_server(connection_string: &str) -> Result<Self, CliError> {
        let context = SqlServerContext::from_ado_string(connection_string)
            .map_err(|error| CliError::UseCase(error.into()))?;

        Ok(Self {
            accounts: Arc::new(SqlServerAccountRepository::new(context)),
            // The audit log has no SQL Server adapter yet, so actions against a
            // real database are recorded to the process log only. Worth having
            // before this is used in anger.
            audit: Arc::new(InMemoryAuditLog::new()),
        })
    }

    /// Adapters held entirely in process.
    ///
    /// Seeded with one account so the implemented path can be exercised without
    /// a database.
    #[must_use]
    pub(crate) fn in_memory() -> Self {
        let seed = Account::new(
            AccountId::new(1),
            AccountName::parse("player01").expect("the seed name is valid"),
            false,
            AccountStatus::Active,
        );

        Self {
            accounts: Arc::new(InMemoryAccountRepository::seeded([seed])),
            audit: Arc::new(InMemoryAuditLog::new()),
        }
    }

    /// The account-banning use case.
    #[must_use]
    pub(crate) fn ban_account(&self) -> BanAccount {
        BanAccount::new(Arc::clone(&self.accounts), Arc::clone(&self.audit))
    }
}
