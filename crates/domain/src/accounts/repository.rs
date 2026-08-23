//! The port: what persistence has to provide for accounts.

use async_trait::async_trait;

use crate::accounts::account::{Account, AccountName};
use crate::error::RepositoryResult;

/// Where accounts live.
///
/// Declared here and implemented in `sro-toolbox-storage`. Kept narrow: an
/// adapter should never have to implement behaviour no use case asks for.
///
/// Async because a real adapter talks to SQL Server over the network, and
/// because the same port must serve a web interface later.
#[async_trait]
pub trait AccountRepository: Send + Sync {
    /// Look an account up by login name.
    ///
    /// Returns `Ok(None)` when no such account exists; that is an ordinary
    /// outcome, not a failure.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::RepositoryError`] if the store cannot be read.
    async fn find_by_name(&self, name: &AccountName) -> RepositoryResult<Option<Account>>;

    /// Persist changes to an account that already exists.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::RepositoryError`] if the store cannot be written.
    async fn save(&self, account: &Account) -> RepositoryResult<()>;
}
