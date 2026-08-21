//! Accounts: the login identity that owns characters.

use serde::{Deserialize, Serialize};

use crate::error::{DomainError, DomainResult};

/// An account's primary key, as the server stores it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AccountId(u32);

impl AccountId {
    /// Wrap a raw identifier.
    #[must_use]
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    /// The underlying identifier.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for AccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A validated login name.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct AccountName(String);

impl AccountName {
    /// Shortest accepted name.
    pub const MIN_LEN: usize = 4;
    /// Longest accepted name.
    pub const MAX_LEN: usize = 16;

    /// Validate and wrap a login name.
    ///
    /// Accepts ASCII letters, digits and underscore. These bounds are the
    /// toolbox's defaults; a server may be stricter.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError`] if the name is empty, outside the length bounds,
    /// or contains an unsupported character.
    pub fn parse(raw: impl Into<String>) -> DomainResult<Self> {
        const FIELD: &str = "account name";
        let raw = raw.into();
        let trimmed = raw.trim();

        if trimmed.is_empty() {
            return Err(DomainError::Empty { field: FIELD });
        }
        if trimmed.len() < Self::MIN_LEN {
            return Err(DomainError::OutOfRange {
                field: FIELD,
                min: Self::MIN_LEN as i64,
                max: Self::MAX_LEN as i64,
                actual: trimmed.len() as i64,
            });
        }
        if trimmed.len() > Self::MAX_LEN {
            return Err(DomainError::TooLong {
                field: FIELD,
                max: Self::MAX_LEN,
                actual: trimmed.len(),
            });
        }
        if let Some(bad) = trimmed
            .chars()
            .find(|c| !c.is_ascii_alphanumeric() && *c != '_')
        {
            return Err(DomainError::InvalidCharacter {
                field: FIELD,
                character: bad,
            });
        }

        Ok(Self(trimmed.to_owned()))
    }

    /// The name as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for AccountName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Why an account was banned.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BanReason(String);

impl BanReason {
    /// Longest accepted reason.
    pub const MAX_LEN: usize = 128;

    /// Validate and wrap a ban reason.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError`] if the reason is blank or too long.
    pub fn parse(raw: impl Into<String>) -> DomainResult<Self> {
        const FIELD: &str = "ban reason";
        let raw = raw.into();
        let trimmed = raw.trim();

        if trimmed.is_empty() {
            return Err(DomainError::Empty { field: FIELD });
        }
        if trimmed.len() > Self::MAX_LEN {
            return Err(DomainError::TooLong {
                field: FIELD,
                max: Self::MAX_LEN,
                actual: trimmed.len(),
            });
        }

        Ok(Self(trimmed.to_owned()))
    }

    /// The reason as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// How long a ban lasts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BanDuration {
    /// Never expires.
    Permanent,
    /// Expires after this many whole days.
    Days(u32),
}

impl BanDuration {
    /// Build a duration in days, rejecting zero.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::OutOfRange`] when `days` is zero — a zero-day ban
    /// is a no-op and almost always a mistake at the call site.
    pub const fn days(days: u32) -> DomainResult<Self> {
        if days == 0 {
            return Err(DomainError::OutOfRange {
                field: "ban duration",
                min: 1,
                max: u32::MAX as i64,
                actual: 0,
            });
        }
        Ok(Self::Days(days))
    }

    /// Whether this ban never expires.
    #[must_use]
    pub const fn is_permanent(self) -> bool {
        matches!(self, Self::Permanent)
    }
}

/// Whether an account may log in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccountStatus {
    /// The account may log in.
    Active,
    /// The account is blocked.
    Banned {
        /// Why it was blocked.
        reason: BanReason,
        /// How long the block lasts.
        duration: BanDuration,
    },
}

/// An account, as the toolbox reasons about it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Account {
    id: AccountId,
    name: AccountName,
    is_gm: bool,
    status: AccountStatus,
}

impl Account {
    /// Assemble an account from parts already validated.
    #[must_use]
    pub const fn new(id: AccountId, name: AccountName, is_gm: bool, status: AccountStatus) -> Self {
        Self {
            id,
            name,
            is_gm,
            status,
        }
    }

    /// The account's identifier.
    #[must_use]
    pub const fn id(&self) -> AccountId {
        self.id
    }

    /// The account's login name.
    #[must_use]
    pub const fn name(&self) -> &AccountName {
        &self.name
    }

    /// Whether the account holds game-master privileges.
    #[must_use]
    pub const fn is_gm(&self) -> bool {
        self.is_gm
    }

    /// Whether the account may currently log in.
    #[must_use]
    pub const fn status(&self) -> &AccountStatus {
        &self.status
    }

    /// Whether the account is currently blocked.
    #[must_use]
    pub const fn is_banned(&self) -> bool {
        matches!(self.status, AccountStatus::Banned { .. })
    }

    /// Block this account.
    ///
    /// Re-banning an already-banned account replaces the existing block, which
    /// is how operators expect "extend this ban" to behave.
    pub fn ban(&mut self, reason: BanReason, duration: BanDuration) {
        self.status = AccountStatus::Banned { reason, duration };
    }

    /// Lift any block on this account.
    ///
    /// Returns `true` if the account was banned and is now active.
    pub fn unban(&mut self) -> bool {
        let was_banned = self.is_banned();
        self.status = AccountStatus::Active;
        was_banned
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    #[case::simple("player01")]
    #[case::underscores("a_b_c_d")]
    #[case::min_length("abcd")]
    #[case::max_length("abcdefghijklmnop")]
    #[case::trims_surrounding_space("  player01  ")]
    fn accepts_valid_account_names(#[case] input: &str) {
        assert!(AccountName::parse(input).is_ok(), "should accept {input:?}");
    }

    #[rstest]
    #[case::empty("", DomainError::Empty { field: "account name" })]
    #[case::whitespace_only("   ", DomainError::Empty { field: "account name" })]
    #[case::too_short("abc", DomainError::OutOfRange { field: "account name", min: 4, max: 16, actual: 3 })]
    #[case::too_long("abcdefghijklmnopq", DomainError::TooLong { field: "account name", max: 16, actual: 17 })]
    #[case::space_inside("bad name", DomainError::InvalidCharacter { field: "account name", character: ' ' })]
    #[case::punctuation("bad-name", DomainError::InvalidCharacter { field: "account name", character: '-' })]
    #[case::sql_quote("rob'ert", DomainError::InvalidCharacter { field: "account name", character: '\'' })]
    fn rejects_invalid_account_names(#[case] input: &str, #[case] expected: DomainError) {
        assert_eq!(AccountName::parse(input), Err(expected));
    }

    #[test]
    fn account_name_is_trimmed_not_merely_accepted() {
        let name = AccountName::parse("  player01  ").expect("valid");
        assert_eq!(name.as_str(), "player01");
    }

    #[rstest]
    #[case::empty("")]
    #[case::whitespace("  ")]
    fn rejects_blank_ban_reasons(#[case] input: &str) {
        assert!(BanReason::parse(input).is_err());
    }

    #[test]
    fn rejects_an_overlong_ban_reason() {
        let long = "x".repeat(BanReason::MAX_LEN + 1);
        assert!(matches!(
            BanReason::parse(long),
            Err(DomainError::TooLong { .. })
        ));
    }

    #[test]
    fn rejects_a_zero_day_ban() {
        assert!(BanDuration::days(0).is_err());
    }

    #[rstest]
    #[case(1)]
    #[case(30)]
    #[case(u32::MAX)]
    fn accepts_a_positive_ban_duration(#[case] days: u32) {
        assert_eq!(BanDuration::days(days), Ok(BanDuration::Days(days)));
    }

    fn sample_account() -> Account {
        Account::new(
            AccountId::new(1),
            AccountName::parse("player01").expect("valid"),
            false,
            AccountStatus::Active,
        )
    }

    #[test]
    fn banning_marks_the_account_blocked() {
        let mut account = sample_account();
        assert!(!account.is_banned());

        account.ban(
            BanReason::parse("botting").expect("valid"),
            BanDuration::Permanent,
        );

        assert!(account.is_banned());
    }

    #[test]
    fn re_banning_replaces_the_existing_block() {
        let mut account = sample_account();
        account.ban(
            BanReason::parse("first").expect("valid"),
            BanDuration::days(1).expect("valid"),
        );
        account.ban(
            BanReason::parse("second").expect("valid"),
            BanDuration::Permanent,
        );

        match account.status() {
            AccountStatus::Banned { reason, duration } => {
                assert_eq!(reason.as_str(), "second");
                assert!(duration.is_permanent());
            }
            AccountStatus::Active => panic!("expected the account to be banned"),
        }
    }

    #[test]
    fn unbanning_reports_whether_it_changed_anything() {
        let mut account = sample_account();
        assert!(!account.unban(), "an active account was not banned");

        account.ban(
            BanReason::parse("botting").expect("valid"),
            BanDuration::Permanent,
        );
        assert!(account.unban(), "a banned account becomes active");
        assert!(!account.is_banned());
    }
}
