# Command reference

```
toolbox [OPTIONS] <AREA> <ACTION>
```

## Global options

| Option | Effect |
|---|---|
| `-o, --output <text\|json>` | Rendering. `json` prints one document, for scripting. Default `text`. |
| `--dry-run` | Accepted, **not yet enforced**; logs a warning. |
| `-v, --verbose` | Log verbosity, repeatable. Logs go to stderr so stdout stays parseable. |
| `-h, --help` / `-V, --version` | |

`TOOLBOX_LOG` overrides verbosity with an `EnvFilter` string, e.g.
`TOOLBOX_LOG=sro_toolbox_application=debug`.

## Status key

| | Meaning |
|---|---|
| ✅ | Implemented end to end |
| ⏳ | Parses and validates, then exits 4 |

## `accounts`

| Command | Status |
|---|---|
| `accounts ban <name> --reason <text> [--days <n>]` | ✅ |
| `accounts unban <name>` | ⏳ |
| `accounts show <name>` | ⏳ |
| `accounts grant-silk <name> --amount <n> [--kind <pool>]` | ⏳ |

Omitting `--days` bans permanently. Banning an already-banned account succeeds
and replaces the existing block, so "extend this ban" behaves as expected; the
output reports whether that happened.

```console
$ toolbox accounts ban player01 --reason "botting" --days 7
Banned player01 for 7 day(s).

$ toolbox accounts ban player01 --reason "botting"
Banned player01 permanently (replacing an existing ban).
```

## Areas not yet present

`accounts` is the only area. Characters, items, spawns, drops, skills, quests,
guilds, sieges, jobs, events, the world, the live server and client patching are
all in scope and each will arrive as its own area with its own use cases. The
survey of what that covers lives in the
[capability catalogue](https://github.com/mahmoudfarouq/pk2/blob/master/docs/server-toolbox-capabilities.md).

## `completions`

```bash
toolbox completions bash > /usr/local/etc/bash_completion.d/toolbox
toolbox completions zsh  > ~/.zfunc/_toolbox
```

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | Storage or rendering failure |
| 2 | Invalid input — rejected by a domain rule |
| 3 | Target not found |
| 4 | Not implemented yet |

## The sandbox account

The in-process adapters are seeded with one active account, `player01`, so the
implemented path can be exercised without a database. Anything else reports
"account not found" and exits 3.
