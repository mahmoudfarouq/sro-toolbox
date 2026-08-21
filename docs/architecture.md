# Architecture

The tool is a Cargo workspace of four crates. The split exists to make one
property structural rather than aspirational: **the user interface is
replaceable.**

## Layers

```
   ┌───────────────────────────────────────────────────────────────┐
   │  crates/cli                              interface adapter    │
   │                                                               │
   │  clap definitions · handlers · rendering · composition root    │
   │  The only crate that knows what a command line is.            │
   └───────────────────────────────┬───────────────────────────────┘
                                   │ calls
                                   ▼
   ┌───────────────────────────────────────────────────────────────┐
   │  crates/application                          use cases        │
   │                                                               │
   │  use_cases/   what the tool can do                            │
   │  ports/       traits describing what it needs to do it        │
   │                                                               │
   │  Knows nothing about clap, SQL Server, or PK2 files.          │
   └──────┬────────────────────────────────────────────▲───────────┘
          │ uses                                       │ implements
          ▼                                            │
   ┌──────────────────────────┐      ┌─────────────────┴───────────┐
   │  crates/domain           │      │  crates/infrastructure      │
   │                          │      │                             │
   │  Types that enforce      │      │  memory/     in-process     │
   │  their own invariants.   │      │  sqlserver/  the real thing │
   │  No I/O. No async.       │      │                             │
   │  No dependencies beyond  │      │  Depends on application for │
   │  serde and thiserror.    │      │  the traits it implements.  │
   └──────────────────────────┘      └─────────────────────────────┘
```

## The dependency rule

Arrows point inwards. `application` declares the traits it needs and
`infrastructure` implements them, which inverts what would otherwise be a
dependency on a database driver. That is the whole trick: it is why the use
cases can be tested without a database, and why a web front end is additive
rather than a rewrite.

Verifiable, not just documented:

```bash
grep -rn "sro_toolbox_infrastructure\|sro_toolbox_cli" crates/application crates/domain --include='*.rs'
# no output
```

## Why each layer exists

**`domain`** holds the rules that are true regardless of how the tool is
invoked. `AccountName::parse` rejects `rob'ert` because the game will not accept
it, not because of SQL injection — though it happens to help there too. Once an
`AccountName` exists, every layer above can stop checking.

**`application`** holds the operations. A use case takes raw strings, validates
them through the domain, calls ports, and returns a serialisable output. It is
the natural unit of behaviour: one use case, one thing an operator can do.

**`infrastructure`** holds the messy parts. Two families today: `memory` for
tests and for a safe sandbox, `sqlserver` as stubs that report
`PortError::Unimplemented`. A PK2 family will join them, most likely built on
[the `pk2` crate](https://github.com/mahmoudfarouq/pk2).

**`cli`** holds the syntax and the rendering, plus the composition root in
`wiring.rs` — the only place that decides which adapter satisfies which port.

## Request flow

```
   toolbox accounts ban player01 --reason "botting" --days 7
        │
        ├─ cli::cli          clap parses into AccountsCommand::Ban { .. }
        │
        ├─ cli::commands     builds BanAccountInput { account_name, reason, days }
        │                    — raw strings, no validation here
        │
        ├─ application       BanAccount::execute
        │                    ├─ AccountName::parse      ─┐
        │                    ├─ BanReason::parse         ├─ domain validates
        │                    ├─ BanDuration::days       ─┘
        │                    ├─ AccountRepository::find_by_name
        │                    ├─ Account::ban            ── domain mutates
        │                    ├─ AccountRepository::save
        │                    └─ AuditLog::record
        │
        ├─ cli::output       renders BanAccountOutput as text or JSON
        │
        └─ exit code         0, or 2/3/4 by failure kind
```

Note the ordering: validation completes before any port is touched. A test
asserts this by pointing the use case at a repository that fails on every read
and checking that invalid input still produces a domain error.

## Adding a second interface

A web panel would be a fifth crate beside `cli`:

1. Add `crates/web` depending on `application` and `infrastructure`.
2. Give it its own composition root.
3. Map HTTP requests to the same use-case inputs.

No change to `domain`, `application`, or `infrastructure`. The use case outputs
already derive `Serialize`, so they are HTTP response bodies as they stand. This
is why the ports are async despite a CLI not needing it — see
[ADR 0002](adr/0002-async-ports.md).

## What is deliberately missing

- Only `accounts` exists, and within it only `ban` is implemented. The other
  three actions parse and validate, then exit 4.
- `domain` contains only accounts, trimmed to keep the skeleton to one path.
  Every other area is additive: a domain module, its ports, its use cases, and a
  variant on `Command`.
- No configuration loading. `SqlServerContext` shows where a connection string
  will arrive.
- `--dry-run` is parsed but not enforced.
