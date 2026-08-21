# Testing

47 tests today, and the shape matters more than the count.

| Crate | Tests | What they cover |
|---|---|---|
| `domain` | 23 | Invariants: what is a valid name, what arithmetic is refused |
| `application` | 10 | Use-case behaviour against fakes |
| `infrastructure` | 5 | Adapters honour their port's contract |
| `cli` | 9 | The built binary: parsing, wiring, rendering, exit codes |

## Table-driven by default

Where a test varies only by input, it is a table. `rstest` gives named cases, so
a failure names itself:

```rust
#[rstest]
#[case::empty("", DomainError::Empty { field: "account name" })]
#[case::too_short("abc", DomainError::OutOfRange { .. })]
#[case::sql_quote("rob'ert", DomainError::InvalidCharacter { character: '\'' })]
fn rejects_invalid_account_names(#[case] input: &str, #[case] expected: DomainError) {
    assert_eq!(AccountName::parse(input), Err(expected));
}
```

`cargo test rejects_invalid_account_names::sql_quote` runs exactly one case, and
a failure reports which case rather than which line.

Two rules that keep tables useful:

- **Name every case.** `#[case::sql_quote(...)]`, not `#[case(...)]`.
- **Assert the specific error, not just `is_err()`.** A test that only checks
  failure will pass when the code fails for the wrong reason.

## Fakes, not mocks

There is no mocking framework. Ports are small enough to implement by hand, and
a hand-written fake can carry exactly the affordance a test needs:

```rust
struct FakeAccounts { account: Mutex<Option<Account>>, fail: bool }

impl FakeAccounts {
    fn holding(account: Account) -> Self { .. }
    fn empty() -> Self { .. }
    fn failing() -> Self { .. }   // every read errors
}
```

`failing()` exists for one specific test: pointing the use case at a repository
that cannot be read, and asserting that invalid input *still* produces a domain
error. That proves validation runs before any port is touched — an ordering
guarantee no amount of happy-path testing would catch.

## The layer decides the test

- **`domain`** — pure functions and invariants. No async, no fakes.
- **`application`** — one use case against fakes. Assert the output, the state
  the fake ended in, and that exactly one audit entry was written.
- **`infrastructure`** — the adapter satisfies its port. The `sqlserver` test
  asserts the stub *reports* `Unimplemented` rather than pretending to succeed.
- **`cli`** — `assert_cmd` over the real binary. Exit codes are asserted by
  number, because scripts depend on them.

## Running

```bash
cargo test --workspace              # everything
cargo test -p sro-toolbox-domain    # one crate
cargo test --workspace -- --nocapture
```

## When adding a use case

Write the test first — the input and output structs are easier to get right when
something is already calling them. Cover, at minimum:

1. The happy path, asserting the output.
2. The target not existing.
3. A port failing.
4. Invalid input, as a table, asserting it fails *before* the port is reached.
5. That the audit entry was written, once.
