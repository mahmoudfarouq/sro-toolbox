# The SQL Server schema, and what we assume about it

What the `sqlserver` adapter expects to find in `SRO_VT_ACCOUNT`, and how
confident we are about each part. This matters because the schema is someone
else's, undocumented, and varies between server-file bases.

Targets **vSRO 1.188**. Compiled from community sources — RaGEZONE, elitepvpers,
vSRO.org, SroArena — and cross-checked between them where possible. There is no
vendor documentation.

## Banning is not a column

The single most important thing about this schema: **an account row carries no
ban flag.** A ban is two rows in two other tables.

```
   TB_User                    the account
      JID  ─────────┐
                    │
                    ├──▶ _Punishment           the record of what was done
                    │       Serial  ◀──┐         survives an unban
                    │                  │
                    └──▶ _BlockedUser  │       what actually stops a login
                            SerialNo ──┘         deleted on unban
                            TimeBegin
                            TimeEnd
```

The login server consults `_BlockedUser`. A row there whose window covers now is
what refuses the login; `_Punishment` is history and explanation.

Three consequences the adapter has to honour:

1. **Reading an account is a join**, and "not banned" is the *absence* of a row
   rather than a false flag.
2. **Writing a ban is two inserts in one transaction.** A `_Punishment` row
   without its `_BlockedUser` counterpart is a ban that was recorded but never
   took effect — which looks, to whoever issued it, exactly like success.
3. **An unban deletes from `_BlockedUser` only.** Erasing the `_Punishment` row
   would erase the fact that the ban ever happened.

## Tables

### `TB_User`

| Column | Type | Confidence | Used for |
|---|---|---|---|
| `JID` | `int IDENTITY` | Confirmed | The account's identity, referenced by both other tables |
| `StrUserID` | `varchar(25)` | Confirmed | The login name |
| `GMrank` | `tinyint` | Confirmed | Non-zero means a game master |

Other columns exist — `password`, `Status`, `Name`, `sec_primary`,
`sec_content`, `regtime`, `reg_ip`, `AccPlayTime` and more — and the adapter
neither reads nor writes them.

> **Note on name length.** `StrUserID` is `varchar(25)`; our `AccountName`
> accepts at most 16 characters. The narrower bound is the toolbox's own, and a
> longer stored name is reported as an error rather than silently truncated.

### `_Punishment`

| Column | Confidence | Written as |
|---|---|---|
| `Serial` | Confirmed (identity) | Returned by `SCOPE_IDENTITY()`, linked from `_BlockedUser` |
| `UserJID` | Confirmed | The account's `JID` |
| `Type` | Column confirmed, **values assumed** | See below |
| `Executor` | Confirmed | `sro-toolbox` |
| `Shard` | Confirmed | `0` |
| `CharName`, `CharInfo`, `PosInfo`, `Guide` | Confirmed | Empty strings — they describe a punishment issued against a character in-world, which this is not |
| `Description` | Confirmed | The operator's reason |
| `RaiseTime`, `BlockStartTime`, `PunishTime` | Confirmed | The moment the ban is issued |
| `BlockEndTime` | Confirmed | When it lifts |
| `Status` | Column confirmed, **values assumed** | `1` |

### `_BlockedUser`

| Column | Confidence | Written as |
|---|---|---|
| `UserJID` | Confirmed | The account's `JID` |
| `UserID` | Confirmed | The login name, duplicated here |
| `Type` | Column confirmed, **values assumed** | See below |
| `SerialNo` | Confirmed | The `_Punishment.Serial` just inserted |
| `TimeBegin`, `TimeEnd` | Confirmed | The window the block covers |

## What we are *not* sure about

Stated plainly, because acting on a wrong guess here silently fails to ban
someone.

- **`Type` values.** The column is confirmed on both tables, and the schema is
  known to distinguish login, chat and trade restrictions. The specific numbers
  are **not** confirmed. The adapter uses `1` for a login block
  (`schema::TYPE_LOGIN_BLOCK`) and exposes no others. If bans issued by this tool
  do not take effect on your server, this constant is the first thing to check.
- **`Status` values.** Same situation; `1` is used for an active punishment.
- **Whether a stored procedure should be preferred.** vSRO ships
  `_RegisterPunishment`, which does roughly what our two inserts do. Calling it
  would inherit any server-side logic we do not know about, at the cost of
  depending on a procedure whose signature varies. We insert directly, which is
  visible and testable; revisiting this is reasonable.

## Conventions we chose

These are ours, not the schema's, and are documented so the choice is visible.

**Permanent bans.** The schema has no "never expires" flag, only a `TimeEnd`. A
permanent ban is written with `TimeEnd = 2999-12-31T23:59:59`
(`schema::PERMANENT_UNTIL`), and reading a block whose end is at or beyond that
maps back to `BanDuration::Permanent`.

**Recovering a ban's length.** Computed from the stored `TimeBegin` and
`TimeEnd`, not from the current time, so reading the same row twice gives the
same answer. A test pins `save` and `find_by_name` against each other, since they
are the only two places that know this convention and would otherwise drift.

**Re-banning replaces.** Saving a ban over an existing one clears the live block
first, so blocks do not stack. This matches the domain, where re-banning replaces
rather than accumulates. The superseded `_Punishment` rows remain as history.

## Testing against a real server

The adapter separates what can be checked without a database from what cannot:
the SQL text, the row-to-domain mapping, and the domain-to-parameters mapping are
all pure and unit-tested. Only the execution needs a live server.

There are no automated integration tests yet. Verifying against a real
`SRO_VT_ACCOUNT` — ideally a throwaway copy — is the outstanding work before
this should be pointed at anything that matters.
