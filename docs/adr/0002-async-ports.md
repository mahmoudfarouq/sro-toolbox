# 0002 — Ports are async, even though the CLI does not need it

## Context

A CLI runs one operation and exits; blocking I/O would be entirely adequate and
simpler. But the stated goal is that the interface is replaceable, and the
plausible second interface is a web panel, where blocking a request thread per
database call is a real problem. The target database is SQL Server, whose Rust
driver (`tiberius`) is async.

## Decision

Port traits are `async fn` via `async-trait`, and the binary uses `tokio`.

## Consequences

**Gained.** A web front end reuses every use case unchanged. Had the ports been
synchronous, adding one would mean either duplicating them or wrapping every call
in `spawn_blocking`.

**Cost.** `async-trait` is required because the ports are used as
`Arc<dyn Port>`, and native `async fn` in traits is not dyn-compatible. That is
one proc-macro dependency and one boxed future per call — irrelevant for a CLI
issuing a handful of queries. Tests must be `#[tokio::test]`. `domain` stays
entirely synchronous, so the async surface is confined to ports and use cases.

**Rejected: sync ports now, convert later.** Async is contagious through every
signature and every test. Converting later is a mechanical but repository-wide
change, and the decision is cheapest to make while there is one use case.
