# 0001 — A four-crate workspace, not one crate with modules

## Context

The tool needs to be usable from a CLI now and plausibly from a web panel later.
Module boundaries inside a single crate express that intent but do not enforce
it: nothing stops `domain` from importing a SQL driver, and the drift is only
visible in review.

## Decision

Four crates: `domain`, `application`, `infrastructure`, `cli`. `application`
declares its ports as traits; `infrastructure` implements them; `cli` is the
composition root.

## Consequences

**Gained.** The dependency rule is checked by the compiler. `application` cannot
reference `infrastructure` because the dependency is not declared, so a
violation is a build failure rather than a review comment. Each crate has its own
dependency list, which makes the intent visible — `domain` pulling in `serde` and
`thiserror` and nothing else says more than any comment.

**Cost.** Four manifests. Adding a type that crosses layers touches more files.
Workspace dependency inheritance (`dep.workspace = true`) keeps versions in one
place, which removes most of the friction but not all.

**Rejected: one crate with `mod domain` etc.** Cheaper today, but the boundary
holds only as long as everyone remembers it exists — including future AI agents
working in this repo, which is exactly when a compiler-enforced rule pays off.
