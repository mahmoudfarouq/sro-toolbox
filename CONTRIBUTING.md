# Contributing

## Before you start

```bash
cargo test --workspace
```

Should be 47 passing. If not, that is the first problem.

## The rule

Dependencies point inwards: `cli → application → domain`, with
`infrastructure` implementing ports that `application` declares. Read
[`AGENTS.md`](AGENTS.md) — it applies to humans equally, it is just written for
whoever forgets.

## Adding a use case

`crates/application/src/use_cases/ban_account.rs` is the worked example. Copy its
shape.

1. **Domain first.** If the operation needs a concept that does not exist yet,
   add it to `crates/domain` as a type that validates itself. Invariants live in
   constructors, so that nothing above has to re-check them.

2. **Port next, if needed.** A new external dependency becomes a narrow trait in
   `crates/application/src/ports/`. Narrow means an adapter never implements
   behaviour no use case asked for.

3. **Test before the use case.** Input and output structs are easier to design
   when something already calls them. See [`docs/testing.md`](docs/testing.md)
   for the required cases.

4. **Then the use case.** Input struct of raw strings, serialisable output
   struct, dependencies as `Arc<dyn Port>`, one `execute`. Validate first, audit
   last.

5. **Adapters.** An in-memory one in `infrastructure/memory` at minimum. A
   `sqlserver` stub returning `PortError::Unimplemented` is acceptable and
   preferable to a half-written query.

6. **Wire it.** Expose it from `crates/cli/src/wiring.rs`, add the syntax to
   `cli.rs`, handle it in `commands/`. Keep the handler thin.

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
