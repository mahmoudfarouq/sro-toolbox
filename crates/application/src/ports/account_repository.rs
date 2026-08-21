//! Reading and writing accounts.

use async_trait::async_trait;
use sro_toolbox_domain::account::{Account, AccountName};

use crate::error::PortResult;

/// Where accounts live.
///
/// Async because a real adapter talks to SQL Server over the network, and
/// because the same ports must serve a web interface later.
#[async_trait]
pub trait AccountRepository: Send + Sync {
    /// Look an account up by login name.
    ///
    /// Returns `Ok(None)` when no such account exists; that is an ordinary
    /// outcome, not a failure.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::PortError`] if the store cannot be read.
    async fn find_by_name(&self, name: &AccountName) -> PortResult<Option<Account>>;

    /// Persist changes to an account that already exists.
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::PortError`] if the store cannot be written.
    async fn save(&self, account: &Account) -> PortResult<()>;
}
