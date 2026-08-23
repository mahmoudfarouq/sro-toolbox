//! Look an account up by login name, including whether it is currently blocked.
//!
//! A ban is not a column on the account. It is a row in `_BlockedUser` whose
//! window covers now, with the human-readable reason living on the matching
//! `_Punishment` row. So reading an account means a left join across all three
//! tables, and "not banned" is the absence of a row rather than a false flag.

use chrono::NaiveDateTime;
use sro_toolbox_domain::accounts::{
    Account, AccountId, AccountName, AccountStatus, BanDuration, BanReason,
};
use sro_toolbox_domain::{RepositoryError, RepositoryResult};
use tiberius::Query;

use crate::sqlserver::{SqlClient, schema, storage_error};

/// The account, plus the block covering now if there is one.
///
/// `TOP 1` with the newest block first: a rewritten ban leaves the superseded
/// rows in place, and the most recent one is the one in force.
fn query() -> String {
    format!(
        "SELECT TOP 1 \
             u.JID, u.StrUserID, u.GMrank, \
             b.TimeBegin, b.TimeEnd, p.Description \
         FROM {user} AS u \
         LEFT JOIN {blocked} AS b \
             ON b.UserJID = u.JID \
            AND b.Type = @P2 \
            AND GETDATE() BETWEEN b.TimeBegin AND b.TimeEnd \
         LEFT JOIN {punishment} AS p \
             ON p.Serial = b.SerialNo \
         WHERE u.StrUserID = @P1 \
         ORDER BY b.TimeEnd DESC",
        user = schema::TB_USER,
        blocked = schema::BLOCKED_USER,
        punishment = schema::PUNISHMENT,
    )
}

/// One row as SQL Server returns it, before it becomes a domain [`Account`].
///
/// Exists so the mapping below is a pure function over plain data and can be
/// tested without a server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AccountRow {
    /// `TB_User.JID`.
    pub(super) jid: i32,
    /// `TB_User.StrUserID`.
    pub(super) user_id: String,
    /// `TB_User.GMrank`; zero means an ordinary player.
    pub(super) gm_rank: u8,
    /// `_BlockedUser.TimeBegin`, when a block covers now.
    pub(super) block_begin: Option<NaiveDateTime>,
    /// `_BlockedUser.TimeEnd`, when a block covers now.
    pub(super) block_end: Option<NaiveDateTime>,
    /// `_Punishment.Description`, the reason shown to an operator.
    pub(super) block_reason: Option<String>,
}

/// Turn a row into a domain [`Account`].
///
/// Pure. Every failure here means the database holds something the domain will
/// not accept, which is worth surfacing rather than papering over.
pub(super) fn to_domain(row: AccountRow) -> RepositoryResult<Account> {
    let name = AccountName::parse(row.user_id.clone()).map_err(|error| {
        RepositoryError::Storage(format!(
            "account {} has a name the domain rejects: {error}",
            row.jid
        ))
    })?;

    let status = match (row.block_begin, row.block_end) {
        (Some(begin), Some(end)) => {
            // A missing or empty Description is not a reason to fail: the block
            // is real either way, and refusing to read the account would be a
            // worse outcome than a placeholder.
            let reason = row
                .block_reason
                .filter(|text| !text.trim().is_empty())
                .unwrap_or_else(|| "no reason recorded".to_owned());
            let reason = BanReason::parse(reason).map_err(|error| {
                RepositoryError::Storage(format!("stored ban reason is unusable: {error}"))
            })?;

            AccountStatus::Banned {
                reason,
                duration: duration_between(begin, end),
            }
        }
        _ => AccountStatus::Active,
    };

    Ok(Account::new(
        AccountId::new(row.jid.unsigned_abs()),
        name,
        row.gm_rank > 0,
        status,
    ))
}

/// Expose the mapping to `save`'s round-trip test, which pins the two halves of
/// the sentinel convention against each other.
#[cfg(test)]
pub(super) fn duration_for_tests(begin: NaiveDateTime, end: NaiveDateTime) -> BanDuration {
    duration_between(begin, end)
}

/// Recover the ban's length from the window the database stores.
///
/// Uses the recorded begin and end rather than the current time, so reading the
/// same row twice gives the same answer.
fn duration_between(begin: NaiveDateTime, end: NaiveDateTime) -> BanDuration {
    let permanent = NaiveDateTime::parse_from_str(schema::PERMANENT_UNTIL, "%Y-%m-%dT%H:%M:%S")
        .expect("the sentinel is a compile-time constant and is covered by a test");

    if end >= permanent {
        return BanDuration::Permanent;
    }

    let days = (end - begin).num_days();
    u32::try_from(days.max(1)).map_or(BanDuration::Permanent, BanDuration::Days)
}

/// Run the lookup.
///
/// # Errors
///
/// Returns [`RepositoryError::Storage`] if the query fails or a row cannot be
/// mapped.
pub(super) async fn execute(
    client: &mut SqlClient,
    name: &AccountName,
) -> RepositoryResult<Option<Account>> {
    let mut statement = Query::new(query());
    statement.bind(name.as_str());
    statement.bind(schema::TYPE_LOGIN_BLOCK);

    let stream = statement
        .query(client)
        .await
        .map_err(|error| storage_error("looking up an account", &error))?;

    let Some(row) = stream
        .into_row()
        .await
        .map_err(|error| storage_error("reading the account row", &error))?
    else {
        return Ok(None);
    };

    let parsed = AccountRow {
        jid: row.get::<i32, _>("JID").unwrap_or_default(),
        user_id: row
            .get::<&str, _>("StrUserID")
            .unwrap_or_default()
            .to_owned(),
        gm_rank: row.get::<u8, _>("GMrank").unwrap_or_default(),
        block_begin: row.get::<NaiveDateTime, _>("TimeBegin"),
        block_end: row.get::<NaiveDateTime, _>("TimeEnd"),
        block_reason: row.get::<&str, _>("Description").map(ToOwned::to_owned),
    };

    to_domain(parsed).map(Some)
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    fn at(text: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(text, "%Y-%m-%dT%H:%M:%S").expect("valid test date")
    }

    fn row() -> AccountRow {
        AccountRow {
            jid: 42,
            user_id: "player01".to_owned(),
            gm_rank: 0,
            block_begin: None,
            block_end: None,
            block_reason: None,
        }
    }

    #[test]
    fn the_query_touches_every_table_a_ban_spans() {
        let sql = query();
        for table in [schema::TB_USER, schema::BLOCKED_USER, schema::PUNISHMENT] {
            assert!(sql.contains(table), "query should join {table}");
        }
        assert!(sql.contains("@P1"), "the name must be a bound parameter");
        assert!(
            sql.contains("@P2"),
            "the block type must be a bound parameter"
        );
    }

    #[test]
    fn the_query_binds_rather_than_interpolates_the_name() {
        // The account name reaches this adapter from an operator's shell.
        assert!(
            !query().contains("StrUserID = '"),
            "no string interpolation"
        );
    }

    #[test]
    fn maps_an_unblocked_account_to_active() {
        let account = to_domain(row()).expect("valid row");

        assert_eq!(account.id().get(), 42);
        assert_eq!(account.name().as_str(), "player01");
        assert!(!account.is_gm());
        assert!(!account.is_banned());
    }

    #[rstest]
    #[case::ordinary_player(0, false)]
    #[case::lowest_gm_rank(1, true)]
    #[case::higher_gm_rank(3, true)]
    fn maps_gm_rank_to_a_flag(#[case] gm_rank: u8, #[case] expected: bool) {
        let account = to_domain(AccountRow { gm_rank, ..row() }).expect("valid row");
        assert_eq!(account.is_gm(), expected);
    }

    #[test]
    fn maps_a_block_to_banned_with_its_reason() {
        let account = to_domain(AccountRow {
            block_begin: Some(at("2026-01-01T00:00:00")),
            block_end: Some(at("2026-01-08T00:00:00")),
            block_reason: Some("botting".to_owned()),
            ..row()
        })
        .expect("valid row");

        match account.status() {
            AccountStatus::Banned { reason, duration } => {
                assert_eq!(reason.as_str(), "botting");
                assert_eq!(*duration, BanDuration::Days(7));
            }
            AccountStatus::Active => panic!("expected the account to be banned"),
        }
    }

    #[rstest]
    #[case::one_day("2026-01-01T00:00:00", "2026-01-02T00:00:00", BanDuration::Days(1))]
    #[case::seven_days("2026-01-01T00:00:00", "2026-01-08T00:00:00", BanDuration::Days(7))]
    #[case::partial_day_rounds_up(
        "2026-01-01T00:00:00",
        "2026-01-01T06:00:00",
        BanDuration::Days(1)
    )]
    #[case::the_sentinel_is_permanent(
        "2026-01-01T00:00:00",
        "2999-12-31T23:59:59",
        BanDuration::Permanent
    )]
    #[case::beyond_the_sentinel_is_permanent(
        "2026-01-01T00:00:00",
        "3999-01-01T00:00:00",
        BanDuration::Permanent
    )]
    fn recovers_the_ban_length_from_the_stored_window(
        #[case] begin: &str,
        #[case] end: &str,
        #[case] expected: BanDuration,
    ) {
        assert_eq!(duration_between(at(begin), at(end)), expected);
    }

    #[rstest]
    #[case::missing(None)]
    #[case::empty(Some(String::new()))]
    #[case::whitespace(Some("   ".to_owned()))]
    fn a_block_without_a_usable_reason_still_reads_as_banned(#[case] stored: Option<String>) {
        // Losing the reason is bad; refusing to read the account would be worse.
        let account = to_domain(AccountRow {
            block_begin: Some(at("2026-01-01T00:00:00")),
            block_end: Some(at("2026-01-02T00:00:00")),
            block_reason: stored,
            ..row()
        })
        .expect("valid row");

        assert!(account.is_banned());
    }

    #[test]
    fn a_half_present_block_window_is_not_a_ban() {
        // A left join with no match yields NULLs in both columns. One without
        // the other means the row is malformed, and inventing a window from it
        // would be worse than reporting the account as active.
        let account = to_domain(AccountRow {
            block_begin: Some(at("2026-01-01T00:00:00")),
            block_end: None,
            ..row()
        })
        .expect("valid row");

        assert!(!account.is_banned());
    }

    #[test]
    fn a_name_the_domain_rejects_is_reported_not_silently_accepted() {
        let error = to_domain(AccountRow {
            user_id: "x".to_owned(),
            ..row()
        })
        .expect_err("should fail");

        assert!(matches!(error, RepositoryError::Storage(_)));
    }
}
