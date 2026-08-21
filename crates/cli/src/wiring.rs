//! The composition root.
//!
//! The single place that decides which adapter satisfies which port. Nothing
//! else in the CLI constructs an adapter, so switching from the in-memory
//! sandbox to a real database — or introducing a web interface that reuses the
//! same use cases — is a change here and nowhere else.

use std::sync::Arc;

use sro_toolbox_application::ports::{AccountRepository, AuditLog};
use sro_toolbox_application::use_cases::BanAccount;
use sro_toolbox_domain::account::{Account, AccountId, AccountName, AccountStatus};
use sro_toolbox_infrastructure::memory::{InMemoryAccountRepository, InMemoryAuditLog};

/// Everything the commands need, already wired.
pub(crate) struct Context {
    accounts: Arc<dyn AccountRepository>,
    audit: Arc<dyn AuditLog>,
}

impl Context {
    /// Build a context backed by in-process adapters.
    ///
    /// This is what runs today. It is seeded with a sample account so the
    /// worked-out command path can be exercised without a database.
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
