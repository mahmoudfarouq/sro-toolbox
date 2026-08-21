# 0004 — thiserror per layer, distinct exit codes

## Context

Failures come from three different places and need different treatment: a
violated domain rule is the operator's mistake, a missing account is an ordinary
outcome, and an unreachable database is an operational problem. A CLI that
reports all three identically forces callers to parse messages.

## Decision

One error enum per layer, each with `thiserror`:

- `DomainError` — an invariant was violated.
- `PortError` — an adapter failed, including `Unimplemented` for stubs.
- `ApplicationError` — wraps both, adds `NotFound`.
- `CliError` — wraps `ApplicationError`, adds `NotImplemented`, and maps to an
  exit code.

Exit codes: `2` invalid input, `3` not found, `4` not implemented, `1` anything
else.

## Consequences

**Gained.** Each layer names failures in its own vocabulary, and `#[from]` makes
the conversions mechanical. Scripts branch on exit codes rather than on message
text. `PortError::Unimplemented` lets the command surface be complete and
navigable while the adapters behind it are stubs — and because it is an error
rather than a `todo!()`, a stub can never be mistaken for working code or crash a
user's session.

**Cost.** Four enums and their conversions, which is more ceremony than a single
`anyhow::Error` in a project this size. Adding a variant means touching the
`exit_code` match — though that is exactly the review prompt wanted, since a new
failure mode should be a deliberate decision about what callers see.

**Rejected: `anyhow` throughout.** Idiomatic for binaries, but it erases the
distinction between kinds of failure, which is the thing being encoded. `anyhow`
would still be reasonable inside a future adapter, where the detail is
diagnostic rather than semantic.
