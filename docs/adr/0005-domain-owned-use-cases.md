# 0005 — Use cases live in the domain, split by subdomain

Supersedes the four-crate arrangement in [0001](0001-layered-workspace.md).

## Context

The original split put entities in `domain` and use cases in a separate
`application` crate. In practice that meant a use case and the entity it exists
to manipulate were never in the same place: banning an account lived in
`application/use_cases/`, while `Account::ban` lived in `domain/account.rs`.
Every change to account behaviour touched two crates, and neither crate answered
"what can this tool do with an account?" on its own.

The layers were also organised by *technical role* — entities here, use cases
there, ports somewhere else — so adding a subdomain meant touching every layer
in parallel rather than adding one coherent unit.

## Decision

Three crates: `domain`, `storage`, `cli`. `application` is gone; its use cases
moved into `domain`.

Within each crate, split by **subdomain** rather than by technical role. A
subdomain owns its entities, its ports and its use cases together, one file per
use case:

```
domain/accounts/          storage/accounts/       cli/accounts/
  account.rs                memory.rs               mod.rs
  repository.rs             sqlserver.rs            ban.rs
  ban_account.rs
```

Repository traits are declared in `domain`, beside the logic that needs them,
and implemented in `storage`.

## Consequences

**Gained.** A subdomain is a unit. Everything about accounts is in
`domain/accounts/`, and adding `items` is three new directories rather than
edits threaded through four crates. One file per use case means a new operation
is a new file, never a new arm in a growing match — the thing that makes large
service modules unpleasant. Reading `domain/accounts/` answers what the tool can
do with an account, which the previous split could not.

**Cost.** `domain` is no longer dependency-free: declaring an async port means
depending on `async-trait`, and that is a real dent in "the domain knows
nothing". A purist arrangement keeps ports in a separate layer precisely to
avoid this. The trade is deliberate — one fewer crate and colocated logic, for
one dependency that never appears in a signature the business rules care about.

The dependency rule survives intact, which is the part that mattered:
`domain` still depends on neither `storage` nor `cli`, still declares what it
needs as traits, and CI still greps to prove it. Three crates enforce that as
well as four did.

**Rejected: a crate per subdomain.** `domain-accounts`, `domain-items`, and so
on would stop one subdomain reaching into another. But subdomains legitimately
reference each other — a character belongs to an account — and a manifest per
subdomain is heavy at this size. Modules are the right granularity for now, and
promoting one to a crate later is mechanical.

**Rejected: keeping `application`.** Defensible with many subdomains and thick
orchestration across them. With one subdomain and one use case it was ceremony
that separated things wanting to be together.
