# sro-toolbox

A command-line toolbox for administering a Silkroad Online private server.

> **Status: early.** Banning an account is built end to end, including a real
> SQL Server adapter against `SRO_VT_ACCOUNT`. Everything else is a deliberate
> stub that reports what is missing rather than pretending to work.
>
> The SQL adapter compiles and its logic is unit-tested, but it has **not yet
> been run against a live server**. See
> [`docs/sqlserver-schema.md`](docs/sqlserver-schema.md) for what it assumes.

```console
# Against a real server
$ toolbox --database "Server=tcp:10.0.0.5,1433;Database=SRO_VT_ACCOUNT;User Id=sa;Password=...;TrustServerCertificate=true" \
      accounts ban player01 --reason "botting" --days 7
Banned player01 for 7 day(s).

# Without --database, an in-memory sandbox: nothing can reach live data
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
   crates/cli  ──────▶  crates/domain  ◀──────  crates/storage
   the only crate       entities, ports         adapters implementing
   that knows clap      and use cases,          those ports: SQL Server,
   exists               split by subdomain      PK2, in-memory
```

`domain` declares the ports it needs and depends on neither of the others.
`storage` implements them; `cli` wires them together. Adding a web front end
means adding one crate beside `cli` and reusing every use case unchanged.

Each crate is split the same way, by subdomain, with one file per use case:

```
domain/accounts/          storage/accounts/       cli/accounts/
  account.rs                memory.rs               mod.rs
  repository.rs             sqlserver.rs            ban.rs
  ban_account.rs
```

See [`docs/architecture.md`](docs/architecture.md) for the full picture,
[`docs/adr/`](docs/adr/) for why each decision was made, and
[`docs/roadmap.md`](docs/roadmap.md) for where this is going.

## Command surface

```
toolbox [--output text|json] [--dry-run] [-v...] <area> <action>

  accounts     ban · unban · show · grant-silk
  completions  <shell>
```

Only `accounts ban` is implemented; the other three actions parse, validate and
exit 4. `accounts` is the only area that exists — characters, items, spawns and
server administration all belong here and will be added as areas of their own,
each with its own use cases. Full listing in
[`docs/commands.md`](docs/commands.md).

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

Without `--database` the tool runs against in-process adapters seeded with one
account, `player01`, so commands can be exercised safely. Supply `--database`
(or `TOOLBOX_DATABASE`) with an ADO connection string to talk to a real server.

## Development

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI runs all three on Linux, macOS and Windows, plus a rustdoc build, a build at
the declared MSRV, and a grep asserting the dependency rule.

[`docs/testing.md`](docs/testing.md) covers the table-driven approach,
[`CONTRIBUTING.md`](CONTRIBUTING.md) covers adding a use case, and
[`CLAUDE.md`](CLAUDE.md) is the guide for AI agents working here.

## License

Licensed under either of [Apache License 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.
