# Architecture decision records

Short notes on decisions that would otherwise be re-litigated. Each records the
context, the choice, and what it costs — the cost is the part worth writing down.

| # | Decision |
|---|---|
| [0001](0001-layered-workspace.md) | A layered workspace, not one crate with modules *(superseded by 0005)* |
| [0002](0002-async-ports.md) | Ports are async, even though the CLI does not need it |
| [0003](0003-clap-for-the-cli.md) | clap with derive |
| [0004](0004-errors-and-exit-codes.md) | thiserror per layer, distinct exit codes |
| [0005](0005-domain-owned-use-cases.md) | Use cases live in the domain, split by subdomain; no application layer |
