# sro-toolbox

A command-line toolbox for administering a Silkroad Online private server.

> **Status: scaffolding.** The architecture, command surface and test approach
> are in place. One use case — banning an account — is built end to end as the
> worked example. Everything else is a deliberate stub that reports what is
> missing rather than pretending to work.

```console
$ toolbox accounts ban player01 --reason "botting" --days 7
Banned player01 for 7 day(s).

$ toolbox --output json accounts ban player01 --reason "botting"
{
  "account_name": "player01",
  "permanent": true,
  "was_already_banned": true
}
```

## Why a workspace

The CLI is *an* interface, not *the* interface — a web panel is a plausible
second one. So the layering is enforced by crate boundaries rather than by
convention:

```
        crates/cli                  the only crate that knows clap exists
             │
             ▼
     crates/application            use cases + the ports they need
             │         ▲
             ▼         │ implements
      crates/domain    crates/infrastructure
      no I/O at all    SQL Server, PK2, in-memory
```

`application` depends on `domain` and on traits it declares itself. It never
depends on `infrastructure` or on `cli`. Adding a web front end means adding one
crate beside `cli` and reusing every use case unchanged.

See [`docs/architecture.md`](docs/architecture.md) for the full picture and
[`docs/adr/`](docs/adr/) for why each decision was made.

## Command surface

```
toolbox [--output text|json] [--dry-run] [-v...] <area> <action>

  accounts     ban · unban · show · grant-silk
  characters   show · set-level · teleport
  items        create · show · grant
  spawns       add · list
  server       status · notice
  completions  <shell>
```

Only `accounts ban` is implemented. The rest parse, validate and exit 4.
Full listing in [`docs/commands.md`](docs/commands.md).

Exit codes are distinct so scripts can branch without parsing messages:

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | Storage or rendering failure |
| 2 | Invalid input |
| 3 | Target not found |
| 4 | Not implemented yet |

## Getting started

```bash
cargo build
cargo run -p sro-toolbox-cli -- accounts ban player01 --reason "testing"
cargo test --workspace
```

Today the tool runs against in-process adapters seeded with one account,
`player01`, so commands can be exercised without a database.

## Development

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI runs all three on Linux, macOS and Windows. `docs/testing.md` covers the
table-driven approach; `CONTRIBUTING.md` covers adding a use case.

## License

Licensed under either of [Apache License 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
