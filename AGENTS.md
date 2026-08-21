# Notes for AI agents

Read this before changing anything. It records the constraints that are easy to
violate and expensive to undo.

## What this is

A CLI toolbox for administering a Silkroad Online private server. Currently
**scaffolding**: one use case is built end to end, everything else is a
deliberate stub.

## The one rule that matters

**Dependencies point inwards. Never outwards.**

```
cli  ──▶  application  ──▶  domain
                 ▲
                 │ implements ports
          infrastructure
```

- `domain` depends on nothing but `serde` and `thiserror`. No I/O, no async, no
  clap, no SQL. If you are tempted to add a dependency here, you are solving the
  problem in the wrong layer.
- `application` depends on `domain` and on the port traits it declares itself in
  `ports/`. It must **never** reference `sro-toolbox-infrastructure` or
  `sro-toolbox-cli`.
- `infrastructure` implements ports. Nothing depends on it except the
  composition root.
- `cli` is the only crate allowed to mention `clap`. Keep handlers thin: parse,
  build input, call use case, render. Logic that appears in a handler is logic a
  web interface would have to duplicate.

A quick check that the arrow has not been reversed:

```bash
grep -rn "sro_toolbox_infrastructure\|sro_toolbox_cli" crates/application crates/domain --include='*.rs'
# must print nothing
```

## Where things go

| You want to add | Put it in |
|---|---|
| A rule about what a valid value is | `crates/domain` — a newtype with a `parse`/`new` that returns `DomainResult` |
| A thing the tool can *do* | `crates/application/src/use_cases/` — input struct, output struct, one `execute` |
| A new external dependency the tool needs | `crates/application/src/ports/` — a narrow trait |
| Talking to SQL Server, PK2 files, the live server | `crates/infrastructure` |
| A new command | `crates/cli/src/cli.rs` for the syntax, `crates/cli/src/commands/` for the handler |
| Deciding which adapter is used | `crates/cli/src/wiring.rs`, and nowhere else |

## Conventions that are enforced, not suggested

- **`missing_docs` is a warning and CI denies warnings.** Every public item
  needs a doc comment. Explain *why*, not what the signature already says.
- **`unsafe_code` is forbidden** workspace-wide.
- **Invariants live in constructors.** If a type exists, its value is valid.
  Validation belongs in `domain`, not in a use case and never in a CLI handler,
  because each interface would duplicate it.
- **Stubs return errors, they do not panic.** Use
  `PortError::Unimplemented` or `CliError::NotImplemented`. Never `todo!()` or
  `unimplemented!()` on a path a user can reach — a stub that panics is
  indistinguishable from a bug.
- **Mutating use cases write an audit entry.** This tool edits live player data.
- **Currency arithmetic is checked.** Silently wrapping a player's balance is the
  worst failure this tool could have.
- **Tests are table-driven where the input varies.** Use `rstest` with named
  cases. See `docs/testing.md`.

## The worked example

`crates/application/src/use_cases/ban_account.rs` is the reference. A new use
case should look like it: dependencies as `Arc<dyn Port>` held in the struct,
raw strings in the input, a serialisable output, validation first, audit last,
and tests using hand-written fakes rather than a mocking framework.

Its test module also shows the pattern worth copying: a fake repository with a
`failing()` constructor, used to prove that validation happens *before* any port
is touched.

## Before you say you are done

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

All three must pass. Do not report success otherwise, and do not silence a lint
to make it pass — fix the code or say why the lint is wrong.

## Domain vocabulary

Terms from the game, used precisely:

- **Shard** — one independent world. A server may run several.
- **Account** vs **character** — an account logs in and owns characters.
- **Silk** — the premium currency, in three non-interchangeable pools:
  purchased, gift, and point.
- **Gold** — the in-game currency.
- **PK2** — the client's archive format. Most content exists twice, once in the
  server's database and once in a PK2 text file, and the two must agree.
- **vSRO** — the server-file base this targets, version 1.188.

## Things that look wrong but are not

- The stub command handlers return `NotImplemented` for every variant. That is
  intentional: the command surface is meant to be complete and navigable before
  the use cases exist.
- `crates/domain` contains only accounts. Characters, items and currencies were
  removed deliberately to keep the skeleton to one worked path; add them back
  alongside the use case that needs them.
- `--dry-run` is parsed but not enforced, and logs a warning saying so. Wiring it
  properly needs a preview-capable adapter.
