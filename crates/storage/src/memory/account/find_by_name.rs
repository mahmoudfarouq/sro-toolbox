//! Look an account up by login name.

use sro_toolbox_domain::RepositoryResult;
use sro_toolbox_domain::accounts::{Account, AccountName};

use super::Store;

/// Fetch the account named `name`, if the store holds one.
pub(super) fn execute(store: &Store, name: &AccountName) -> RepositoryResult<Option<Account>> {
    Ok(store
        .lock()
        .expect("lock is never poisoned: no panics while held")
        .get(name)
        .cloned())
}

#[cfg(test)]
mod tests {
    use super::super::{InMemoryAccountRepository, tests::account};
    use super::*;
    use sro_toolbox_domain::accounts::AccountRepository;

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
}
