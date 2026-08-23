//! Persist an account's block state.
//!
//! Banning is not an `UPDATE`. The account row carries no ban column: a block is
//! a row in `_Punishment` recording what happened, plus a row in `_BlockedUser`
//! that the login server actually consults. So saving reconciles the stored
//! block against the account's [`AccountStatus`]:
//!
//! ```text
//!   status is Banned   ->  clear any live block, then insert punishment
//!                          + blocked-user rows for the new one
//!   status is Active   ->  clear any live block
//! ```
//!
//! Both inserts happen in one transaction. A `_Punishment` row without its
//! `_BlockedUser` counterpart is a ban that was recorded but never took effect —
//! the worst possible outcome for this tool, because it looks like it worked.

use chrono::{Duration, NaiveDateTime};
use sro_toolbox_domain::accounts::{Account, AccountStatus, BanDuration};
use sro_toolbox_domain::{RepositoryError, RepositoryResult};
use tiberius::Query;

use crate::sqlserver::{SqlClient, schema, storage_error};

/// Who the audit trail credits a ban to when the tool issues it.
const EXECUTOR: &str = "sro-toolbox";

/// Record the punishment and return the identity assigned to it.
fn insert_punishment_sql() -> String {
    format!(
        "INSERT INTO {punishment} \
             (UserJID, Type, Executor, Shard, CharName, CharInfo, PosInfo, Guide, \
              Description, RaiseTime, BlockStartTime, BlockEndTime, PunishTime, Status) \
         VALUES (@P1, @P2, @P3, 0, '', '', '', '', @P4, @P5, @P5, @P6, @P5, @P7); \
         SELECT CAST(SCOPE_IDENTITY() AS INT) AS Serial",
        punishment = schema::PUNISHMENT,
    )
}

/// Put the block in force.
fn insert_blocked_user_sql() -> String {
    format!(
        "INSERT INTO {blocked} (UserJID, UserID, Type, SerialNo, TimeBegin, TimeEnd) \
         VALUES (@P1, @P2, @P3, @P4, @P5, @P6)",
        blocked = schema::BLOCKED_USER,
    )
}

/// Lift any block currently in force.
///
/// Deletes only from `_BlockedUser`. The `_Punishment` row stays: it is the
/// history of what was done, and an unban should not erase that.
fn delete_blocked_user_sql() -> String {
    format!(
        "DELETE FROM {blocked} WHERE UserJID = @P1 AND Type = @P2",
        blocked = schema::BLOCKED_USER,
    )
}

/// Everything the two inserts need, derived from the domain.
///
/// Built by a pure function so the arithmetic — particularly what a permanent
/// ban's end date becomes — is testable without a server.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BanParameters {
    /// `TB_User.JID` of the account being blocked.
    pub(super) jid: i32,
    /// `TB_User.StrUserID`, which `_BlockedUser` stores alongside the id.
    pub(super) user_id: String,
    /// The reason, stored as `_Punishment.Description`.
    pub(super) description: String,
    /// When the block starts.
    pub(super) begin: NaiveDateTime,
    /// When the block ends; the sentinel for a permanent ban.
    pub(super) end: NaiveDateTime,
}

/// Work out the rows a ban becomes.
///
/// `now` is passed in rather than read, so the result is deterministic.
pub(super) fn ban_parameters(
    account: &Account,
    reason: &str,
    duration: BanDuration,
    now: NaiveDateTime,
) -> RepositoryResult<BanParameters> {
    let end = match duration {
        BanDuration::Permanent => {
            NaiveDateTime::parse_from_str(schema::PERMANENT_UNTIL, "%Y-%m-%dT%H:%M:%S").map_err(
                |error| {
                    RepositoryError::Storage(format!("permanent-ban sentinel is unusable: {error}"))
                },
            )?
        }
        BanDuration::Days(days) => now
            .checked_add_signed(Duration::days(i64::from(days)))
            .ok_or_else(|| {
                RepositoryError::Storage(format!("a ban of {days} days runs past the end of time"))
            })?,
    };

    Ok(BanParameters {
        jid: i32::try_from(account.id().get()).map_err(|_| {
            RepositoryError::Storage(format!("account id {} does not fit JID", account.id()))
        })?,
        user_id: account.name().to_string(),
        description: reason.to_owned(),
        begin: now,
        end,
    })
}

/// Reconcile the stored block state with `account`.
///
/// # Errors
///
/// Returns [`RepositoryError::Storage`] if any statement fails. The whole
/// reconciliation runs in one transaction, so a failure leaves no partial ban.
pub(super) async fn execute(
    client: &mut SqlClient,
    account: &Account,
    now: NaiveDateTime,
) -> RepositoryResult<()> {
    let jid = i32::try_from(account.id().get()).map_err(|_| {
        RepositoryError::Storage(format!("account id {} does not fit JID", account.id()))
    })?;

    client
        .simple_query("BEGIN TRANSACTION")
        .await
        .map_err(|error| storage_error("beginning a transaction", &error))?;

    let outcome = reconcile(client, account, jid, now).await;

    let closing = if outcome.is_ok() {
        "COMMIT TRANSACTION"
    } else {
        "ROLLBACK TRANSACTION"
    };
    client
        .simple_query(closing)
        .await
        .map_err(|error| storage_error("closing the transaction", &error))?;

    outcome
}

/// The body of the transaction, separated so both exits close it exactly once.
async fn reconcile(
    client: &mut SqlClient,
    account: &Account,
    jid: i32,
    now: NaiveDateTime,
) -> RepositoryResult<()> {
    // An existing block is cleared first either way. Re-banning replaces rather
    // than stacks, which matches how the domain treats it.
    let mut clear = Query::new(delete_blocked_user_sql());
    clear.bind(jid);
    clear.bind(schema::TYPE_LOGIN_BLOCK);
    clear
        .execute(client)
        .await
        .map_err(|error| storage_error("clearing the existing block", &error))?;

    let AccountStatus::Banned { reason, duration } = account.status() else {
        return Ok(());
    };

    let parameters = ban_parameters(account, reason.as_str(), *duration, now)?;

    let mut punishment = Query::new(insert_punishment_sql());
    punishment.bind(parameters.jid);
    punishment.bind(schema::TYPE_LOGIN_BLOCK);
    punishment.bind(EXECUTOR);
    punishment.bind(parameters.description.as_str());
    punishment.bind(parameters.begin);
    punishment.bind(parameters.end);
    punishment.bind(schema::STATUS_ACTIVE);

    let serial: i32 = punishment
        .query(client)
        .await
        .map_err(|error| storage_error("recording the punishment", &error))?
        .into_row()
        .await
        .map_err(|error| storage_error("reading the punishment id", &error))?
        .and_then(|row| row.get::<i32, _>("Serial"))
        .ok_or_else(|| {
            RepositoryError::Storage(
                "the punishment row was inserted but returned no id, so the block cannot be linked"
                    .to_owned(),
            )
        })?;

    let mut blocked = Query::new(insert_blocked_user_sql());
    blocked.bind(parameters.jid);
    blocked.bind(parameters.user_id.as_str());
    blocked.bind(schema::TYPE_LOGIN_BLOCK);
    blocked.bind(serial);
    blocked.bind(parameters.begin);
    blocked.bind(parameters.end);
    blocked
        .execute(client)
        .await
        .map_err(|error| storage_error("putting the block in force", &error))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use sro_toolbox_domain::accounts::{AccountId, AccountName};

    use super::*;

    fn at(text: &str) -> NaiveDateTime {
        NaiveDateTime::parse_from_str(text, "%Y-%m-%dT%H:%M:%S").expect("valid test date")
    }

    fn account() -> Account {
        Account::new(
            AccountId::new(42),
            AccountName::parse("player01").expect("valid"),
            false,
            AccountStatus::Active,
        )
    }

    #[test]
    fn the_punishment_insert_names_every_column_the_schema_requires() {
        let sql = insert_punishment_sql();
        for column in [
            "UserJID",
            "Type",
            "Executor",
            "Shard",
            "CharName",
            "CharInfo",
            "PosInfo",
            "Guide",
            "Description",
            "RaiseTime",
            "BlockStartTime",
            "BlockEndTime",
            "PunishTime",
            "Status",
        ] {
            assert!(sql.contains(column), "insert should set {column}");
        }
    }

    #[test]
    fn the_punishment_insert_returns_its_identity() {
        // Without the id there is nothing to point _BlockedUser.SerialNo at.
        assert!(insert_punishment_sql().contains("SCOPE_IDENTITY"));
    }

    #[test]
    fn the_blocked_user_insert_names_every_column_the_schema_requires() {
        let sql = insert_blocked_user_sql();
        for column in [
            "UserJID",
            "UserID",
            "Type",
            "SerialNo",
            "TimeBegin",
            "TimeEnd",
        ] {
            assert!(sql.contains(column), "insert should set {column}");
        }
    }

    #[test]
    fn unbanning_leaves_the_punishment_history_alone() {
        let sql = delete_blocked_user_sql();
        assert!(sql.contains(schema::BLOCKED_USER));
        assert!(
            !sql.contains(schema::PUNISHMENT),
            "an unban must not erase the record that the ban happened"
        );
    }

    #[test]
    fn every_statement_binds_its_values() {
        for sql in [
            insert_punishment_sql(),
            insert_blocked_user_sql(),
            delete_blocked_user_sql(),
        ] {
            assert!(
                sql.contains("@P1"),
                "should bind rather than interpolate: {sql}"
            );
            assert!(!sql.contains("'{"), "no formatted values in SQL: {sql}");
        }
    }

    #[test]
    fn a_permanent_ban_ends_at_the_sentinel() {
        let parameters = ban_parameters(
            &account(),
            "botting",
            BanDuration::Permanent,
            at("2026-01-01T12:00:00"),
        )
        .expect("valid");

        assert_eq!(parameters.begin, at("2026-01-01T12:00:00"));
        assert_eq!(parameters.end, at(schema::PERMANENT_UNTIL));
    }

    #[rstest]
    #[case::one_day(1, "2026-01-02T12:00:00")]
    #[case::seven_days(7, "2026-01-08T12:00:00")]
    #[case::a_year(365, "2027-01-01T12:00:00")]
    fn a_timed_ban_ends_that_many_days_out(#[case] days: u32, #[case] expected: &str) {
        let parameters = ban_parameters(
            &account(),
            "botting",
            BanDuration::Days(days),
            at("2026-01-01T12:00:00"),
        )
        .expect("valid");

        assert_eq!(parameters.end, at(expected));
    }

    #[test]
    fn a_ban_long_enough_to_overflow_is_refused_not_wrapped() {
        let error = ban_parameters(
            &account(),
            "botting",
            BanDuration::Days(u32::MAX),
            at("2026-01-01T12:00:00"),
        )
        .expect_err("should fail");

        assert!(matches!(error, RepositoryError::Storage(_)));
    }

    #[test]
    fn parameters_carry_the_identity_and_name_both_tables_need() {
        let parameters = ban_parameters(
            &account(),
            "botting",
            BanDuration::Permanent,
            at("2026-01-01T12:00:00"),
        )
        .expect("valid");

        assert_eq!(parameters.jid, 42);
        assert_eq!(parameters.user_id, "player01");
        assert_eq!(parameters.description, "botting");
    }

    #[test]
    fn a_ban_and_its_read_back_agree_on_the_duration() {
        // What save writes, find_by_name must recover. These two functions are
        // the only places that know the sentinel convention, so pinning them
        // against each other is what stops them drifting apart.
        let now = at("2026-01-01T00:00:00");
        for duration in [
            BanDuration::Permanent,
            BanDuration::Days(1),
            BanDuration::Days(30),
        ] {
            let written = ban_parameters(&account(), "botting", duration, now).expect("valid");
            let read_back =
                super::super::find_by_name::duration_for_tests(written.begin, written.end);
            assert_eq!(read_back, duration, "round trip for {duration:?}");
        }
    }
}
