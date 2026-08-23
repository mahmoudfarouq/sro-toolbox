//! Accounts in `SRO_VT_ACCOUNT`.

mod find_by_name;
mod save;

use async_trait::async_trait;
use sro_toolbox_domain::RepositoryResult;
use sro_toolbox_domain::accounts::{Account, AccountName, AccountRepository};

use super::SqlServerContext;

/// Accounts read from and written to `SRO_VT_ACCOUNT`.
#[derive(Debug, Clone)]
pub struct SqlServerAccountRepository {
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
    async fn find_by_name(&self, name: &AccountName) -> RepositoryResult<Option<Account>> {
        let mut client = self.context.connect().await?;
        find_by_name::execute(&mut client, name).await
    }

    async fn save(&self, account: &Account) -> RepositoryResult<()> {
        let mut client = self.context.connect().await?;
        save::execute(&mut client, account, chrono::Utc::now().naive_utc()).await
    }
}
