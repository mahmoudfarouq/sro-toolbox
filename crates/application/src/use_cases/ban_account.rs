//! Block an account from logging in.

use std::sync::Arc;

use serde::Serialize;
use sro_toolbox_domain::account::{AccountName, BanDuration, BanReason};

use crate::error::{ApplicationError, ApplicationResult};
use crate::ports::{AccountRepository, AuditEntry, AuditLog};

/// What the caller supplies. Raw strings, because validation is the domain's
/// job and doing it here would duplicate it per interface.
#[derive(Debug, Clone)]
pub struct BanAccountInput {
    /// The account to block.
    pub account_name: String,
    /// Why it is being blocked.
    pub reason: String,
    /// How many days the block lasts, or `None` for permanent.
    pub days: Option<u32>,
}

/// What the caller gets back.
///
/// Serializable so an interface can render it as a table, as JSON, or as an
/// HTTP response without the use case knowing which.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BanAccountOutput {
    /// The account that was blocked.
    pub account_name: String,
    /// Whether the block never expires.
    pub permanent: bool,
    /// Whether the account was already blocked before this ran.
    pub was_already_banned: bool,
}

/// Blocks an account.
///
/// Holds only the ports it needs — an account store and an audit log — so it
/// can be exercised against fakes without a database.
pub struct BanAccount {
    accounts: Arc<dyn AccountRepository>,
    audit: Arc<dyn AuditLog>,
}

impl BanAccount {
    /// Build the use case from its dependencies.
    #[must_use]
    pub fn new(accounts: Arc<dyn AccountRepository>, audit: Arc<dyn AuditLog>) -> Self {
        Self { accounts, audit }
    }

    /// Block the account named in `input`.
    ///
    /// Banning an already-banned account succeeds and replaces the existing
    /// block, which is how operators expect "extend this ban" to behave. The
    /// output says whether that happened.
    ///
    /// # Errors
    ///
    /// Returns [`ApplicationError::Domain`] if the name, reason or duration is
    /// invalid, [`ApplicationError::NotFound`] if no such account exists, or
    /// [`ApplicationError::Port`] if the store or audit log fails.
    pub async fn execute(&self, input: BanAccountInput) -> ApplicationResult<BanAccountOutput> {
        let name = AccountName::parse(input.account_name)?;
        let reason = BanReason::parse(input.reason)?;
        let duration = match input.days {
            Some(days) => BanDuration::days(days)?,
            None => BanDuration::Permanent,
        };

        let mut account =
            self.accounts
                .find_by_name(&name)
                .await?
                .ok_or_else(|| ApplicationError::NotFound {
                    kind: "account",
                    identifier: name.to_string(),
                })?;

        let was_already_banned = account.is_banned();
        account.ban(reason.clone(), duration);
        self.accounts.save(&account).await?;

        self.audit
            .record(AuditEntry {
                action: "accounts.ban",
                target: name.to_string(),
                detail: format!(
                    "reason={:?} duration={}",
                    reason.as_str(),
                    match duration {
                        BanDuration::Permanent => "permanent".to_owned(),
                        BanDuration::Days(days) => format!("{days}d"),
                    }
                ),
            })
            .await?;

        Ok(BanAccountOutput {
            account_name: name.to_string(),
            permanent: duration.is_permanent(),
            was_already_banned,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use rstest::rstest;
    use sro_toolbox_domain::account::{Account, AccountId, AccountStatus};

    use super::*;
    use crate::error::{PortError, PortResult};

    /// A repository holding one optional account, with a switch for failure.
    #[derive(Default)]
    struct FakeAccounts {
        account: Mutex<Option<Account>>,
        fail: bool,
    }

    impl FakeAccounts {
        fn holding(account: Account) -> Self {
            Self {
                account: Mutex::new(Some(account)),
                fail: false,
            }
        }

        fn empty() -> Self {
            Self::default()
        }

        fn failing() -> Self {
            Self {
                account: Mutex::new(None),
                fail: true,
            }
        }

        fn saved(&self) -> Option<Account> {
            self.account.lock().expect("not poisoned").clone()
        }
    }

    #[async_trait::async_trait]
    impl AccountRepository for FakeAccounts {
        async fn find_by_name(&self, _name: &AccountName) -> PortResult<Option<Account>> {
            if self.fail {
                return Err(PortError::Storage("cannot reach the database".into()));
            }
            Ok(self.account.lock().expect("not poisoned").clone())
        }

        async fn save(&self, account: &Account) -> PortResult<()> {
            *self.account.lock().expect("not poisoned") = Some(account.clone());
            Ok(())
        }
    }

    #[derive(Default)]
    struct FakeAudit {
        entries: Mutex<Vec<AuditEntry>>,
    }

    #[async_trait::async_trait]
    impl AuditLog for FakeAudit {
        async fn record(&self, entry: AuditEntry) -> PortResult<()> {
            self.entries.lock().expect("not poisoned").push(entry);
            Ok(())
        }
    }

    fn account(status: AccountStatus) -> Account {
        Account::new(
            AccountId::new(1),
            AccountName::parse("player01").expect("valid"),
            false,
            status,
        )
    }

    fn input() -> BanAccountInput {
        BanAccountInput {
            account_name: "player01".into(),
            reason: "botting".into(),
            days: None,
        }
    }

    #[tokio::test]
    async fn bans_an_active_account() {
        let accounts = Arc::new(FakeAccounts::holding(account(AccountStatus::Active)));
        let audit = Arc::new(FakeAudit::default());
        let use_case = BanAccount::new(accounts.clone(), audit.clone());

        let output = use_case.execute(input()).await.expect("should succeed");

        assert_eq!(
            output,
            BanAccountOutput {
                account_name: "player01".into(),
                permanent: true,
                was_already_banned: false,
            }
        );
        assert!(accounts.saved().expect("saved").is_banned());
        assert_eq!(audit.entries.lock().expect("not poisoned").len(), 1);
    }

    #[tokio::test]
    async fn reports_when_the_account_was_already_banned() {
        let already = account(AccountStatus::Banned {
            reason: BanReason::parse("earlier").expect("valid"),
            duration: BanDuration::Days(1),
        });
        let use_case = BanAccount::new(
            Arc::new(FakeAccounts::holding(already)),
            Arc::new(FakeAudit::default()),
        );

        let output = use_case.execute(input()).await.expect("should succeed");

        assert!(output.was_already_banned);
    }

    #[tokio::test]
    async fn records_exactly_one_audit_entry() {
        let audit = Arc::new(FakeAudit::default());
        let use_case = BanAccount::new(
            Arc::new(FakeAccounts::holding(account(AccountStatus::Active))),
            audit.clone(),
        );

        use_case.execute(input()).await.expect("should succeed");

        let entries = audit.entries.lock().expect("not poisoned");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].action, "accounts.ban");
        assert_eq!(entries[0].target, "player01");
    }

    #[tokio::test]
    async fn fails_when_the_account_does_not_exist() {
        let use_case = BanAccount::new(
            Arc::new(FakeAccounts::empty()),
            Arc::new(FakeAudit::default()),
        );

        let error = use_case.execute(input()).await.expect_err("should fail");

        assert!(matches!(
            error,
            ApplicationError::NotFound {
                kind: "account",
                ..
            }
        ));
    }

    #[tokio::test]
    async fn propagates_a_storage_failure() {
        let use_case = BanAccount::new(
            Arc::new(FakeAccounts::failing()),
            Arc::new(FakeAudit::default()),
        );

        let error = use_case.execute(input()).await.expect_err("should fail");

        assert!(matches!(error, ApplicationError::Port(_)));
    }

    #[rstest]
    #[case::blank_name("", "botting", None)]
    #[case::short_name("ab", "botting", None)]
    #[case::name_with_quote("rob'ert", "botting", None)]
    #[case::blank_reason("player01", "", None)]
    #[case::zero_day_ban("player01", "botting", Some(0))]
    #[tokio::test]
    async fn rejects_invalid_input_before_touching_the_store(
        #[case] account_name: &str,
        #[case] reason: &str,
        #[case] days: Option<u32>,
    ) {
        // The repository fails on any read, so reaching it would surface as a
        // Port error. A Domain error proves validation happened first.
        let use_case = BanAccount::new(
            Arc::new(FakeAccounts::failing()),
            Arc::new(FakeAudit::default()),
        );

        let error = use_case
            .execute(BanAccountInput {
                account_name: account_name.into(),
                reason: reason.into(),
                days,
            })
            .await
            .expect_err("should fail");

        assert!(
            matches!(error, ApplicationError::Domain(_)),
            "expected a domain error, got {error:?}"
        );
    }
}
