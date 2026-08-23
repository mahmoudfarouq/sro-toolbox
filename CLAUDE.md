# Developing in this repository

Guidance for Claude Code and other AI agents. Written for agents, but the
constraints apply to everyone; humans may prefer
[`CONTRIBUTING.md`](CONTRIBUTING.md), which covers the same ground as a workflow.

## What this is

A CLI toolbox for administering a Silkroad Online private server — bans, items,
spawns, silk, and eventually the rest of what a server operator does by hand
against SQL Server and the client's PK2 archives today.

**Status: scaffolding.** One use case is built end to end. Everything else is a
deliberate stub. Do not mistake the breadth of the roadmap for the breadth of
the code.

## Orientation

```
crates/
├── domain/          business logic + entities, split by subdomain
│   └── src/
│       ├── error.rs           DomainError · RepositoryError · UseCaseError
│       ├── audit.rs           the audit port, cross-cutting
│       └── accounts/          one subdomain
│           ├── account.rs         entities and value objects
│           ├── repository.rs      the port persistence must satisfy
│           └── ban_account.rs     one use case, one file
│
├── storage/         adapters implementing the domain's ports
│   └── src/                    adapter, then subdomain, then one file per operation
│       ├── memory/account/{find_by_name,save}.rs
│       ├── memory/audit/record.rs
│       └── sqlserver/
│           ├── schema.rs           table names and magic numbers, in one place
│           └── account/{find_by_name,save}.rs
│
└── cli/             the interface; the only crate that knows clap exists
    └── src/
        ├── cli.rs             root options + top-level areas
        ├── wiring.rs          composition root — the only place adapters are chosen
        └── accounts/          mirrors domain/accounts
            ├── mod.rs             subcommand enum + dispatch
            └── ban.rs             one action, one file
```

Every layer is split the same way, by subdomain. Adding `items` means
`domain/items/`, `storage/items/`, `cli/items/` — never a new top-level concept.

## The one rule

**Dependencies point inwards.**

```
cli ──▶ domain ◀── storage
```

`domain` declares the ports it needs and depends on neither of the others.
`storage` implements those ports. `cli` wires them together. That inversion is
why a use case can be tested without a database and why a web front end would be
an additional crate rather than a rewrite.

CI enforces it with a grep; you can run the same check:

```bash
grep -rn "sro_toolbox_storage\|sro_toolbox_cli" crates/domain --include='*.rs'
# must print nothing
```

There is no separate application layer. Use cases live in `domain` alongside the
entities they operate on — see
[`docs/adr/0005-domain-owned-use-cases.md`](docs/adr/0005-domain-owned-use-cases.md)
for why, and what it costs.

## Where things go

| You want to add | Put it in |
|---|---|
| A rule about what a valid value is | `domain/<subdomain>/<entity>.rs` — a newtype whose constructor returns `DomainResult` |
| A thing the tool can *do* | `domain/<subdomain>/<verb>_<noun>.rs` — one file per use case |
| A new external dependency the logic needs | `domain/<subdomain>/repository.rs`, or a new port module |
| Talking to SQL Server, PK2 files, the live server | `storage/<adapter>/<subdomain>/<operation>.rs` |
| A new command | `cli/<subdomain>/mod.rs` for the syntax, `cli/<subdomain>/<action>.rs` for the handler |
| Deciding which adapter is used | `cli/src/wiring.rs`, and nowhere else |
| A structural decision | An ADR in `docs/adr/` |

## Conventions that are enforced, not suggested

- **`missing_docs` warns and CI denies warnings.** Every public item needs a doc
  comment. Say *why*, not what the signature already says.
- **`unsafe_code` is forbidden** workspace-wide.
- **Invariants live in constructors.** If a type exists, its value is valid.
  Validation belongs in `domain`, never in a CLI handler — each interface would
  otherwise duplicate it.
- **Stubs return errors, never panic.** `RepositoryError::Unimplemented` or
  `CliError::NotImplemented`. Never `todo!()` on a path a user can reach: a stub
  that panics is indistinguishable from a bug.
- **Mutating use cases write an audit entry.** This tool edits live player data.
- **Currency arithmetic is checked.** Silently wrapping a player's balance is the
  worst failure this tool could have.
- **Tables for varying input.** `rstest` with named cases. See
  [`docs/testing.md`](docs/testing.md).
- **Handlers stay thin.** Build input, call use case, render. Logic in a handler
  is logic a web interface would have to duplicate.

## The worked example

`crates/domain/src/accounts/ban_account.rs` is the reference. A new use case
should look like it:

- dependencies as `Arc<dyn Port>` held in the struct
- raw strings in the input struct — validation is the domain's job, done once
- a `Serialize` output struct, so any interface can render it
- validate first, audit last
- hand-written fakes in tests, no mocking framework

Its test module shows the pattern worth copying: a fake repository with a
`failing()` constructor, used to prove validation happens *before* any port is
touched. That is an ordering guarantee no amount of happy-path testing catches.

## Commands

```bash
cargo build --workspace
cargo test --workspace
cargo run -p sro-toolbox-cli -- accounts ban player01 --reason "testing"

cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
```

All three checks must pass before you claim to be done. Do not silence a lint to
get there — fix the code, or say why the lint is wrong here.

The in-memory adapters are seeded with one account, `player01`, so the
implemented path can be exercised without a database. Anything else exits 3.

## Domain vocabulary

- **Shard** — one independent world. A server may run several.
- **Account** vs **character** — an account logs in and owns characters.
- **Silk** — the premium currency, in three non-interchangeable pools:
  purchased, gift, point.
- **Gold** — the in-game currency.
- **PK2** — the client's archive format. Most content exists **twice**, once in
  the server's database and once in a PK2 text file, and the two must agree.
  This is the single most important constraint on the whole project; see
  [`docs/roadmap.md`](docs/roadmap.md).
- **vSRO 1.188** — the server-file base this targets. Other bases differ.

## Things that look wrong but are not

- `accounts` is the only area, and only `ban` within it works. The other three
  actions return `NotImplemented` on purpose — enough to show the shape of a stub
  without four areas of them.
- `domain` contains only accounts. Characters, items, currencies and world
  geometry were removed deliberately to keep the skeleton to one worked path. Add
  each back alongside the use case that needs it, not in advance.
- `domain` depends on `async-trait`, which a strictly pure domain would not. That
  is the deliberate cost of owning the ports; ADR 0005 explains.
- `--dry-run` is parsed but not enforced, and logs a warning saying so. Wiring it
  properly needs a preview-capable adapter.
- The SQL Server adapter has never run against a live server. Its logic is
  unit-tested and it compiles, but the `Type` and `Status` constants in
  `sqlserver/schema.rs` are **assumed**, not confirmed. Read
  [`docs/sqlserver-schema.md`](docs/sqlserver-schema.md) before trusting it.
- `storage` splits by adapter first, then subdomain — the reverse of `domain`
  and `cli`. That is deliberate: an adapter is the unit that gets swapped.

## Scope

The roadmap is broad and the code is not. When asked to add something, add the
thin vertical slice — entity, use case, adapter, command — rather than a wide
layer of entities with nothing using them. One working path beats five
half-built ones, and the structure is designed to make slices cheap.
