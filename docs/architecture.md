# Architecture

The tool is a Cargo workspace of four crates. The split exists to make one
property structural rather than aspirational: **the user interface is
replaceable.**

## Layers

```
   ┌───────────────────────────────────────────────────────────────┐
   │  crates/cli                              interface adapter    │
   │                                                               │
   │  cli.rs        root options, top-level areas                  │
   │  wiring.rs     composition root                               │
   │  accounts/     one module per area, one file per action       │
   │                                                               │
   │  The only crate that knows what a command line is.            │
   └───────────────────────────┬───────────────────────────────────┘
                               │ calls
                               ▼
   ┌───────────────────────────────────────────────────────────────┐
   │  crates/domain              business logic and entities       │
   │                                                               │
   │  error.rs      DomainError · RepositoryError · UseCaseError   │
   │  audit.rs      the audit port, cross-cutting                  │
   │  accounts/     one module per subdomain:                      │
   │     account.rs      entities and value objects                │
   │     repository.rs   the port persistence must satisfy         │
   │     ban_account.rs  one use case, one file                    │
   │                                                               │
   │  Knows nothing about clap, SQL Server, or PK2 files.          │
   └───────────────────────────▲───────────────────────────────────┘
                               │ implements the ports
   ┌───────────────────────────┴───────────────────────────────────┐
   │  crates/storage                          storage adapters     │
   │                                                               │
   │  accounts/memory.rs      in-process, for tests and a sandbox  │
   │  accounts/sqlserver.rs   the real target, currently stubs     │
   │  audit/memory.rs                                              │
   └───────────────────────────────────────────────────────────────┘
```

## Split by subdomain, not by technical role

Every crate is divided the same way. Adding `items` means `domain/items/`,
`storage/items/`, `cli/items/` — three coherent additions rather than edits
threaded through layers organised by what kind of code they hold.

One file per use case. A new operation is a new file, never a new arm in a
growing match.

There is no separate application layer: use cases live in `domain` beside the
entities they manipulate. See [ADR 0005](adr/0005-domain-owned-use-cases.md) for
the reasoning and its cost.

## Why each layer exists

**`domain`** holds the rules that are true regardless of how the tool is
invoked. `AccountName::parse` rejects `rob'ert` because the game will not accept
it, not because of SQL injection — though it happens to help there too. Once an
`AccountName` exists, every layer above can stop checking.

`domain` also holds the operations. A use case takes raw strings, validates them
through its own entities, calls ports, and returns a serialisable output. It is
the natural unit of behaviour: one use case, one thing an operator can do.

**`storage`** holds the messy parts. Two families today: `memory` for tests and
for a safe sandbox, `sqlserver` as stubs that report
`RepositoryError::Unimplemented`. A PK2 family will join them, most likely built
on [the `pk2` crate](https://github.com/mahmoudfarouq/pk2).

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
        ├─ domain            BanAccount::execute
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

1. Add `crates/web` depending on `domain` and `storage`.
2. Give it its own composition root.
3. Map HTTP requests to the same use-case inputs.

No change to `domain` or `storage`. The use case outputs
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
