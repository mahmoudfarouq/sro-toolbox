//! Names and magic numbers from the `SRO_VT_ACCOUNT` schema.
//!
//! Collected here rather than inlined so that the assumptions this adapter makes
//! about someone else's database are visible in one place, and so a server whose
//! schema differs has one file to look at.
//!
//! See `docs/sqlserver-schema.md` for provenance and confidence.

/// The account table: `JID int IDENTITY`, `StrUserID varchar(25)`,
/// `GMrank tinyint`, among others.
pub const TB_USER: &str = "TB_User";

/// The punishment history. One row per punishment ever issued, keyed by an
/// identity column, referenced by [`BLOCKED_USER`] through its `SerialNo`.
pub const PUNISHMENT: &str = "_Punishment";

/// The active blocks. The login server consults this; a row here with `TimeEnd`
/// in the future is what actually stops a login.
pub const BLOCKED_USER: &str = "_BlockedUser";

/// The punishment type meaning "blocked from logging in".
///
/// The schema also carries chat and trade restrictions under other values. Only
/// the login block is confirmed, so only it is exposed; see
/// `docs/sqlserver-schema.md`.
pub const TYPE_LOGIN_BLOCK: i32 = 1;

/// `_Punishment.Status` for a block that is in force.
pub const STATUS_ACTIVE: i32 = 1;

/// When a permanent ban's `TimeEnd` is set to.
///
/// The schema has no "never expires" flag, so a permanent ban is expressed as an
/// end date far enough out to be effectively permanent. Reading a block whose
/// end is at or beyond this maps back to
/// [`BanDuration::Permanent`](sro_toolbox_domain::accounts::BanDuration::Permanent).
pub const PERMANENT_UNTIL: &str = "2999-12-31T23:59:59";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_permanent_sentinel_parses_as_a_date() {
        let parsed = chrono::NaiveDateTime::parse_from_str(PERMANENT_UNTIL, "%Y-%m-%dT%H:%M:%S");
        assert!(
            parsed.is_ok(),
            "{PERMANENT_UNTIL:?} must be a valid datetime"
        );
    }

    #[test]
    fn the_permanent_sentinel_is_far_enough_out() {
        let parsed = chrono::NaiveDateTime::parse_from_str(PERMANENT_UNTIL, "%Y-%m-%dT%H:%M:%S")
            .expect("valid");
        assert!(
            parsed > chrono::Utc::now().naive_utc(),
            "a permanent ban must not have already expired"
        );
    }
}
