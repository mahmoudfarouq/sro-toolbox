//! Accounts held in a map.

mod find_by_name;
mod save;

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use sro_toolbox_domain::RepositoryResult;
use sro_toolbox_domain::accounts::{Account, AccountName, AccountRepository};

/// The shared state each operation acts on.
pub(super) type Store = Mutex<HashMap<AccountName, Account>>;

/// An account store backed by a map.
#[derive(Debug, Default)]
pub struct InMemoryAccountRepository {
    accounts: Store,
}

impl InMemoryAccountRepository {
    /// An empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A store pre-populated with `accounts`.
    #[must_use]
    pub fn seeded(accounts: impl IntoIterator<Item = Account>) -> Self {
        let map = accounts
            .into_iter()
            .map(|account| (account.name().clone(), account))
            .collect();
        Self {
            accounts: Mutex::new(map),
        }
    }
}

#[async_trait]
impl AccountRepository for InMemoryAccountRepository {
    async fn find_by_name(&self, name: &AccountName) -> RepositoryResult<Option<Account>> {
        find_by_name::execute(&self.accounts, name)
    }

    async fn save(&self, account: &Account) -> RepositoryResult<()> {
        save::execute(&self.accounts, account)
    }
}

#[cfg(test)]
mod tests {
    use sro_toolbox_domain::accounts::{AccountId, AccountStatus, BanDuration, BanReason};

    use super::*;

    pub(super) fn account(name: &str) -> Account {
        Account::new(
            AccountId::new(1),
            AccountName::parse(name).expect("valid"),
            false,
            AccountStatus::Active,
        )
    }

    #[tokio::test]
    async fn save_then_find_round_trips() {
        let repository = InMemoryAccountRepository::seeded([account("player01")]);
        let name = AccountName::parse("player01").expect("valid");

        let mut stored = repository
            .find_by_name(&name)
            .await
            .expect("no failure")
            .expect("seeded");
        stored.ban(
            BanReason::parse("botting").expect("valid"),
            BanDuration::Permanent,
        );
        repository.save(&stored).await.expect("no failure");

        assert!(
            repository
                .find_by_name(&name)
                .await
                .expect("no failure")
                .expect("still there")
                .is_banned()
        );
    }
}
