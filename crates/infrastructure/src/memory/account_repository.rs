//! Accounts held in a map.

use std::collections::HashMap;
use std::sync::Mutex;

use async_trait::async_trait;
use sro_toolbox_application::error::PortResult;
use sro_toolbox_application::ports::AccountRepository;
use sro_toolbox_domain::account::{Account, AccountName};

/// An account store backed by a map.
#[derive(Debug, Default)]
pub struct InMemoryAccountRepository {
    accounts: Mutex<HashMap<AccountName, Account>>,
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
    async fn find_by_name(&self, name: &AccountName) -> PortResult<Option<Account>> {
        Ok(self
            .accounts
            .lock()
            .expect("lock is never poisoned: no panics while held")
            .get(name)
            .cloned())
    }

    async fn save(&self, account: &Account) -> PortResult<()> {
        self.accounts
            .lock()
            .expect("lock is never poisoned: no panics while held")
            .insert(account.name().clone(), account.clone());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use sro_toolbox_domain::account::{AccountId, AccountStatus, BanDuration, BanReason};

    use super::*;

    fn account(name: &str) -> Account {
        Account::new(
            AccountId::new(1),
            AccountName::parse(name).expect("valid"),
            false,
            AccountStatus::Active,
        )
    }

    #[tokio::test]
    async fn finds_a_seeded_account() {
        let repository = InMemoryAccountRepository::seeded([account("player01")]);
        let name = AccountName::parse("player01").expect("valid");

        assert!(
            repository
                .find_by_name(&name)
                .await
                .expect("no failure")
                .is_some()
        );
    }

    #[tokio::test]
    async fn a_missing_account_is_none_not_an_error() {
        let repository = InMemoryAccountRepository::new();
        let name = AccountName::parse("nobody01").expect("valid");

        assert!(
            repository
                .find_by_name(&name)
                .await
                .expect("no failure")
                .is_none()
        );
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
