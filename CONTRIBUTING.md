# Contributing

## Before you start

```bash
cargo test --workspace
```

Should be 47 passing. If not, that is the first problem.

## The rule

Dependencies point inwards: `cli → domain ← storage`. The domain declares the
ports it needs; storage implements them. Read [`CLAUDE.md`](CLAUDE.md) — written
for agents, but it applies to humans equally.

## Adding a use case

`crates/domain/src/accounts/ban_account.rs` is the worked example. Copy its shape.

1. **Entities first.** If the operation needs a concept that does not exist yet,
   add it to `crates/domain/<subdomain>/` as a type that validates itself.
   Invariants live in constructors, so that nothing above has to re-check them.

2. **Port next, if needed.** A new external dependency becomes a narrow trait
   beside the logic that needs it, in `crates/domain/<subdomain>/`. Narrow means
   an adapter never implements behaviour no use case asked for.

3. **Test before the use case.** Input and output structs are easier to design
   when something already calls them. See [`docs/testing.md`](docs/testing.md)
   for the required cases.

4. **Then the use case**, in its own file named for the operation. Input struct
   of raw strings, serialisable output struct, dependencies as `Arc<dyn Port>`,
   one `execute`. Validate first, audit last.

5. **Adapters.** An in-memory one in `storage/<subdomain>/memory.rs` at minimum.
   A `sqlserver.rs` stub returning `RepositoryError::Unimplemented` is acceptable
   and preferable to a half-written query.

6. **Wire it.** Expose it from `crates/cli/src/wiring.rs`, add the variant to the
   area's subcommand enum, and give the action its own file in
   `crates/cli/src/<subdomain>/`. Keep the handler thin.

7. **Document it.** Update [`docs/commands.md`](docs/commands.md), and flip its
   status marker.

## Checks

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

CI runs these on Linux, macOS and Windows, plus a rustdoc build with warnings
denied, a build at the declared MSRV, and a grep asserting the dependency rule.

Do not silence a lint to make CI pass. Fix the code, or explain in the PR why the
lint is wrong here.

## Commits

Conventional-ish prefixes: `feat:`, `fix:`, `refactor:`, `docs:`, `chore:`,
`test:`, `ci:`. Explain *why* in the body; the diff already shows what.

## Decisions

Anything structural gets an ADR in [`docs/adr/`](docs/adr/). Record the cost, not
only the benefit — the cost is what a future reader needs in order to revisit it.
