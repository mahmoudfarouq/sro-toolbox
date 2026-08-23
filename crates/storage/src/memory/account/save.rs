//! Persist an account.

use sro_toolbox_domain::RepositoryResult;
use sro_toolbox_domain::accounts::Account;

use super::Store;

/// Insert or replace `account`, keyed by its login name.
pub(super) fn execute(store: &Store, account: &Account) -> RepositoryResult<()> {
    store
        .lock()
        .expect("lock is never poisoned: no panics while held")
        .insert(account.name().clone(), account.clone());
    Ok(())
}

#[cfg(test)]
mod tests {
    use sro_toolbox_domain::accounts::{AccountName, AccountRepository, BanDuration, BanReason};

    use super::super::{InMemoryAccountRepository, tests::account};

    #[tokio::test]
    async fn saving_replaces_the_stored_account() {
        let repository = InMemoryAccountRepository::seeded([account("player01")]);
        let name = AccountName::parse("player01").expect("valid");

        let mut banned = account("player01");
        banned.ban(
            BanReason::parse("botting").expect("valid"),
            BanDuration::Permanent,
        );
        repository.save(&banned).await.expect("no failure");

        assert!(
            repository
                .find_by_name(&name)
                .await
                .expect("no failure")
                .expect("present")
                .is_banned()
        );
    }
}
