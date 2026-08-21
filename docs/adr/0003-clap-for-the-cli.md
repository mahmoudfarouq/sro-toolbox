# 0003 — clap with derive

## Context

The command surface is a two-level noun-verb tree — `accounts ban`,
`items create` — with global flags, subcommand help, and shell completions.

## Decision

`clap` v4 with the `derive` feature, plus `clap_complete` for completion
scripts. Confined to `crates/cli`.

## Consequences

**Gained.** The command tree is a set of enums, so adding a command means adding
a variant and the compiler finds every match that needs updating — the
exhaustiveness check is what keeps the stub handlers honest. Help text comes from
doc comments, which means it cannot drift from the code. `clap_complete`
generates completions from the same definition rather than a hand-maintained
script. It is also the ecosystem default, which matters for a project meant to
attract contributors.

**Cost.** clap and its dependencies are the bulk of the build. Derive macros
push some errors into macro expansion, which reads poorly. The enums live in a
crate that no other crate depends on, so none of this leaks.

**Rejected: `argh` or hand-rolled parsing.** Smaller, but no completions, weaker
help, and no benefit given clap is confined to one crate.
