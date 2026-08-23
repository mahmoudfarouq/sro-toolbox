# Roadmap

Everything this toolbox could eventually do, and where it currently stands.

The survey below is deliberately broad: it maps the whole space rather than
committing to it. Nothing here is scheduled, and plenty of it may never be worth
building. It exists so that decisions about what to build next are made against
the full picture instead of against whatever is top of mind.

Originally compiled in the [`pk2`](https://github.com/mahmoudfarouq/pk2)
repository while surveying the format; moved here, where it belongs.

## Where we are

| | |
|---|---|
| **Built** | `accounts ban`, end to end — domain entities, use case, in-memory storage, CLI |
| **Scaffolded** | `accounts unban`, `accounts show`, `accounts grant-silk` — parse and validate, then exit 4 |
| **Stubbed** | The SQL Server adapter reports `Unimplemented` rather than pretending to work |
| **Not started** | Everything else below |

## What the first real milestone probably is

Not more commands. The tool cannot touch a real server yet, so the highest-value
work is the SQL Server adapter behind the one use case that already exists —
`accounts ban` against `SRO_VT_ACCOUNT`. That turns the whole vertical slice
from a demonstration into something usable, and it settles the questions every
later adapter inherits: connection handling, configuration, transactions, and
how a failed write is reported.

After that, breadth is cheap: each new area is a domain module, its ports, its
use cases, and a variant on `Command`.

---

> **Provenance.** Compiled from community sources: the
> [SRO Developing Docs](https://srodevs.gitbook.io/docs) knowledge base,
> RaGEZONE and elitepvpers development forums, the vSRO.org and SroArena
> communities, and the published feature sets of existing tools
> ([SROUltra](https://sroultra.com/), SRO Database Command Center, Z-Filter,
> and [JellyBitz's tool collection](https://github.com/JellyBitz)). There is no
> vendor documentation for any of this; everything below is reverse-engineered.
>
> Details are stated as they apply to **vSRO 1.188**, the most widely used
> server-file base. Other bases and caps differ.

---

## Contents

- [The four surfaces](#the-four-surfaces)
- [1. Accounts and authentication](#1-accounts-and-authentication)
- [2. Characters and progression](#2-characters-and-progression)
- [3. Items and inventory](#3-items-and-inventory)
- [4. Currencies and economy](#4-currencies-and-economy)
- [5. Shops and item mall](#5-shops-and-item-mall)
- [6. NPCs](#6-npcs)
- [7. Monsters and spawns](#7-monsters-and-spawns)
- [8. Drops and loot](#8-drops-and-loot)
- [9. Skills and masteries](#9-skills-and-masteries)
- [10. Quests](#10-quests)
- [11. Guilds, unions and fortress war](#11-guilds-unions-and-fortress-war)
- [12. Job system and trade routes](#12-job-system-and-trade-routes)
- [13. PvP, arenas and events](#13-pvp-arenas-and-events)
- [14. World, regions and maps](#14-world-regions-and-maps)
- [15. Dungeons and instances](#15-dungeons-and-instances)
- [16. Server configuration and operations](#16-server-configuration-and-operations)
- [17. Client build and distribution](#17-client-build-and-distribution)
- [18. Logging and analytics](#18-logging-and-analytics)
- [19. Security and anti-cheat](#19-security-and-anti-cheat)
- [20. Web and external integration](#20-web-and-external-integration)
- [21. Automation and scheduling](#21-automation-and-scheduling)
- [Cross-cutting problems](#cross-cutting-problems)

---

## The four surfaces

Every capability below acts on one or more of four distinct surfaces. This
matters because it determines whether a change is live, needs a server restart,
or needs players to re-download the client.

```
   ┌──────────────────────────────────────────────────────────────────┐
   │  1. DATABASE            Microsoft SQL Server                     │
   │     SRO_VT_ACCOUNT      accounts, silk, bans, shard list         │
   │     SRO_VT_SHARD        the world: chars, items, refs, spawns    │
   │     SRO_VT_LOG          gateway/account-level events             │
   │     SRO_VT_SHARDLOG     in-game events: kills, trades, items     │
   │     SRO_VT_SHARD_INIT   pristine template for resets             │
   │                                                                  │
   │     Mostly offline; some tables are read once at boot.            │
   ├──────────────────────────────────────────────────────────────────┤
   │  2. CLIENT ARCHIVES     PK2 files shipped to players             │
   │     Media.pk2           text data, config, UI strings, shops     │
   │     Data.pk2            models, skill data, effects              │
   │     Map.pk2             terrain, navmesh, region geometry        │
   │     Music.pk2 · Particles.pk2                                    │
   │                                                                  │
   │     Requires a client patch to reach players.                    │
   ├──────────────────────────────────────────────────────────────────┤
   │  3. LIVE SERVER         running processes                        │
   │     GlobalManager · MachineManager · FarmManager                 │
   │     DownloadServer · GatewayServer · AgentServer                 │
   │     SR_ShardManager · SR_GameServer · certification server       │
   │                                                                  │
   │     GM console commands, _ExeGameServer queue, DLL injection.     │
   │     Takes effect immediately.                                    │
   ├──────────────────────────────────────────────────────────────────┤
   │  4. NETWORK             the wire between client and server       │
   │     Proxy / packet filter sitting in front of AgentServer        │
   │                                                                  │
   │     Where exploit filtering and protocol-level features live.     │
   └──────────────────────────────────────────────────────────────────┘
```

The hard constraint that shapes everything: **surfaces 1 and 2 must agree.**
Most content lives twice — once in a `SRO_VT_SHARD` reference table for the
server, once in a `Media.pk2` text file for the client. Change one without the
other and the client desyncs, shows blank names, or crashes.

---

## 1. Accounts and authentication

| Capability | Surface |
|---|---|
| Create, edit, delete accounts | DB (`TB_User`) |
| Grant GM status and privilege tiers | DB |
| Ban and unban, temporary or permanent, with reason and duration | DB (`_Punishment`, `_BlockedUser`) |
| Block by IP, and IP allow-lists for staff | DB / network |
| Reset passwords; separate web and game passwords | DB |
| Change email, secret question | DB |
| Lock an account to a machine (HWID binding) | Network / client |
| Failed-login throttling and lockout | Web / network |
| View and revoke active sessions, force-disconnect | Live |
| Bulk operations across accounts | DB |
| Captcha (IBUV) toggle at login | Live config |

## 2. Characters and progression

| Capability | Surface |
|---|---|
| Create, rename, delete, undelete characters | DB (`_Char`) |
| Set level, EXP, SP, skill points, stat points | DB |
| Set gold carried, HP/MP, Hwan level, berserk state | DB |
| Set strength/intellect distribution | DB |
| Move a character to any coordinate or region | DB / live |
| Resize inventory, storage and guild storage | DB |
| Grant or reset masteries and skills | DB |
| Set or clear job level and job EXP | DB |
| Assign titles, honour rank, honour points | DB |
| Set PK level and murderer flags, clear PK penalty | DB |
| Restore a deleted or rolled-back character | DB + logs |
| Configure what brand-new characters receive: gear, level, gold, stats, skills | DB |
| Auto-equip on level-up, without downgrading better gear | DB / live |
| Transfer a character between accounts | DB |
| Inspect any character's full state | DB |

## 3. Items and inventory

| Capability | Surface |
|---|---|
| Define new items end to end | DB (`_RefObjCommon`, `_RefObjItem`) + `itemdata_*.txt` |
| Edit stats, degree, rarity, required level, gender, race | DB + client |
| Set stack size, trade and drop flags, sell price | DB |
| Set durability, repair cost, and "cannot be repaired" | DB |
| Grant items directly into a character's inventory or storage | DB / live |
| Set plus level (enhancement) on an existing item | DB (`_Items`) |
| Set or reroll magic options (blue stats) | DB (`_BindingOptionWithItem`) |
| Set the "white" attribute/variance bits | DB |
| Trace one item's whole history by `Serial64` | Logs |
| Search for items across every character, chest and stall | DB |
| Bulk-fix a property across a whole item class | DB |
| Delete or confiscate an item | DB |
| Restore items lost to a failed alchemy or a bug | DB + logs |
| Define avatars, costumes, pets, transport, grab pets | DB + client |
| Configure alchemy: success rates per plus level, stone effects, downgrade and destruction rules | DB |
| Set up devil spirits, and the immortal/lucky/steady stone behaviour | DB |

## 4. Currencies and economy

vSRO exposes distinct payment types, and a toolbox would touch all of them:

| Code | Currency |
|---|---|
| 1 | Gold |
| 2 | Silk |
| 4 | Gift silk |
| 16 | Points |
| 32 | Honor points |
| 64 / 128 / 256 / 512 | Copper / Iron / Silver / Gold coin |
| 1024 | Arena coin |

| Capability | Surface |
|---|---|
| Add, remove or set silk per account | DB (`SK_Silk`) |
| Distinguish purchased silk, gift silk and point silk | DB |
| Bulk silk grants across a population or a filtered subset | DB |
| Silk-per-hour-online accrual | Live / scheduled |
| Add or set gold, on a character or in storage | DB |
| Manage coin currencies and their earn rates | DB |
| Read the full silk transaction ledger | Logs |
| Set global gold-drop rates and per-mob multipliers | DB |
| Track and cap total currency in circulation | DB / analytics |
| Detect duplication by reconciling ledgers against holdings | Logs |

## 5. Shops and item mall

| Capability | Surface |
|---|---|
| Add, remove and reprice items in any NPC shop | DB (`_RefShopGoods`, `_RefPricePolicyOfItem`) + client |
| Create shop tabs and tab groups | DB (`_RefShopTab`, `_RefShopTabGroup`) |
| Build package items and their scrap contents | DB (`_RefPackageItem`, `_RefScrapOfPackageItem`) |
| Attach a shop to an NPC | DB (`_RefShop`, `_RefShopGroup`) |
| Configure item-mall listings and their currency | DB |
| Flag items as new / hot / recommended on the mall page | DB |
| Set per-item purchase limits and stock | DB |
| Time-limited and seasonal mall offers | DB / scheduled |
| Player stalls: allow, restrict, tax, or read their contents | DB / live |
| Web-based mall and web stall, gold or silk priced | Web + DB |

## 6. NPCs

| Capability | Surface |
|---|---|
| Add a new NPC with model, name and dialogue | DB + `characterdata_*.txt`, `textdata_object.txt` |
| Place, move or remove an NPC | DB (`Tab_RefNest`) |
| Hide an NPC without deleting it | DB |
| Attach shops, storage, stables, blacksmith or guild functions | DB |
| Attach teleport function and destination list | DB (`_RefTeleport*`) + `teleportdata.txt` |
| Attach quests | DB |
| Script conversation and interaction behaviour | Lua |
| Set NPC-specific tax rates | DB |

## 7. Monsters and spawns

The spawn system is a three-table chain — a toolbox almost certainly wants to
present it as one operation rather than three:

```
   Tab_RefTactics  ──▶  Tab_RefNest  ──▶  Tab_RefHive
   what it does /       where it is /     the spawn group /
   AI, aggression       how many          respawn timing
        │
        └──▶ _RefObjChar / _RefObjCommon   what it *is*
```

| Capability | Surface |
|---|---|
| Define a new monster: model, level, HP, EXP, stats, resistances | DB + `characterdata_*.txt` |
| Create unique / boss monsters with custom stats | DB + client |
| Place spawn points anywhere, with count and radius | DB |
| Set respawn delay and spawn caps | DB |
| Set AI and aggression profile, patrol behaviour | DB |
| Assign champion / giant / elite variants and their rates | DB |
| Scale a monster's level, HP or EXP in bulk across a region | DB |
| Schedule a unique to appear at fixed times | Scheduled + live |
| Announce a unique spawn or death server-wide | Live |
| Spawn or kill a monster live, at will | Live (GM console) |
| Track unique kill history and who landed the blow | Logs |

## 8. Drops and loot

| Capability | Surface |
|---|---|
| Assign items to a monster's drop table | DB (`_RefMonster_AssignedItemDrop`) |
| Set drop probability per item (`DropRatio`, 1 = 100%) | DB |
| Set min and max drop quantity | DB |
| Random-drop pools and their independent rates | DB (`_RefMonster_AssignedItemRndDrop`) |
| Drop groups shared across many monsters | DB (`_RefDropItemGroup`) |
| Equipment and rare-equipment class selection | DB (`_RefDropClassSel_Equip`, `_RefDropClassSel_RareEquip`) |
| Gold drop amount and rate | DB |
| Coin drop configuration | DB |
| Global drop-rate multiplier, and per-region or per-mob overrides | DB |
| Disable all drops from a given monster | DB |
| Rare / SOX drop rates by degree | DB |
| Unique chests: reward on a rare kill, solo or whole party | DB / live |
| Preview what a monster will actually drop, as a probability table | Read-only tooling |

## 9. Skills and masteries

| Capability | Surface |
|---|---|
| Define new skills and their effects | DB (`_RefSkill`) + `skilldata_*enc.txt` |
| Edit damage, cost, cooldown, cast time, duration, range | DB + client |
| Configure skill effects through param codes | DB |
| Raise the skill cap and add higher-level skill tiers | DB + client |
| Add or remove masteries, set mastery caps | DB |
| Grant a full CH or EU skill set to a character | DB |
| Reset a character's skills, refunding SP | DB |
| Job-specific and race-specific skills | DB + client |
| Buff and debuff definitions, including GM buffs | DB / live |
| Item-triggered skills — scrolls, grants, potions | DB |

## 10. Quests

| Capability | Surface |
|---|---|
| Define a quest, its stages and conditions | DB (`_RefQuest`) + client |
| Set rewards: items, gold, EXP, SP, silk | DB (`_RefQuestReward`, `_RefQuestRewardItems`) |
| Attach a quest to an NPC and set prerequisites | DB |
| Daily and repeatable quests | DB |
| Starter quest chains for new players | DB |
| Job quests and level-gated quests | DB |
| Read a character's quest progress | DB |
| Force-complete or reset a quest for a player | DB |
| Quest completion analytics | Logs |

## 11. Guilds, unions and fortress war

| Capability | Surface |
|---|---|
| Create, rename, disband a guild | DB (`_Guild`) |
| Edit roster, ranks, permissions | DB (`_GuildMember`) |
| Set guild level and guild points | DB |
| Manage guild storage | DB |
| Create and dissolve unions/alliances | DB |
| Set guild and union member caps | DB |
| Register guilds for a siege | DB |
| Assign, transfer or reset fortress ownership | DB (`_Fortress`) |
| Set fortress tax rate and read tax income | DB |
| Schedule fortress war windows | Scheduled |
| Configure fortress war rewards | DB |
| Read siege event history | Logs |
| Multiple fortresses — Jangan, Hotan, Bandit, Baghdad | DB + client |

## 12. Job system and trade routes

| Capability | Surface |
|---|---|
| Configure trader, hunter and thief roles | DB |
| Set job level caps and job EXP curves | DB |
| Define trade goods, their buy and sell prices per town | DB |
| Configure transport pets and their capacity | DB |
| Set trade route profitability and distance bonuses | DB |
| Job-based reward payouts: gold, silk, job silk, stones | DB / scheduled |
| Job suits, job-only items and job-only skills | DB + client |
| Job-war scheduling and job-based PvP rules | Scheduled / live |
| Restrict scrolls and items while in job mode | DB / network filter |
| Job ranking boards | DB / web |
| Add new trade routes and towns | DB + client |

## 13. PvP, arenas and events

Existing panels ship large libraries of these; the space is well explored.

| Capability | Surface |
|---|---|
| Free-for-all and last-man-standing events | Live / scripted |
| Team, party, guild and job-versus-job tournaments | Live / scripted |
| Capture the flag | Live / scripted |
| Battle arena, in 8v8, party, guild and job variants | DB + live |
| Survival and invasion events (e.g. boss invasions) | Live / scripted |
| Arena coin earning and arena shops | DB |
| Reward tables per win, loss and draw | DB |
| Event scheduling with kill limits and time windows | Scheduled |
| Automatic announcements and countdowns | Live |
| Event-only maps, buffs and item restrictions | DB + live |
| Lucky-party and random-reward mechanics | Live / scripted |
| Manual event control — start, stop, force a winner | Live |
| Consistent-hashing of teams, spectator handling | Live |

## 14. World, regions and maps

| Capability | Surface |
|---|---|
| Enable regions that ship disabled | DB (`_RefRegion`, `_RefRegionBindAssocServer`) + `refregion.txt` |
| Add whole new continents (Baghdad, Jupiter, and similar) | DB + `Map.pk2` |
| Bind regions to a specific game server | DB |
| Add teleport links and gate networks | DB + `teleportdata.txt`, `teleportbuilding.txt` |
| Set teleport costs and level requirements | DB |
| Reverse-return scroll destinations | DB |
| Place world objects and structures | `Map.pk2` |
| Edit collision and walkability | `Map.pk2` navmesh (`.nvm`) |
| Edit terrain height and texture | `Map.pk2` (`.m`, `JMXVMAPM`, `JMXVMAPT`) |
| Define safe zones, no-PvP zones, and level-gated areas | DB |
| Weather and time of day, globally or per region | Live |
| Minimap and world-map imagery to match | `Media.pk2` |

## 15. Dungeons and instances

| Capability | Surface |
|---|---|
| Create a dungeon: region, entrance, exit | DB + `Map.pk2` |
| Instance per party or per player | DB / live |
| Entry requirements: level, item, quest, cooldown | DB |
| Per-dungeon spawn tables and boss stages | DB |
| Per-dungeon drop tables and completion rewards | DB |
| Timers, wipe conditions and lockouts | DB / live |
| Elite rooms and paid-entry rooms | DB |
| Occupancy and clear-rate analytics | Logs |

## 16. Server configuration and operations

| Capability | Surface |
|---|---|
| Start, stop and monitor each server process | Live |
| Set shard name, capacity and the visible server list | DB + config |
| Configure IPs and ports for gateway, agent, download | Config files |
| Set concurrent-user caps and the login queue | Config |
| Multiple shards and multi-machine deployment | Config |
| Global rate multipliers: EXP, SP, gold, drops, job | DB |
| Level cap, degree cap and mastery cap | DB + client |
| Open, close or restrict logins; maintenance mode | Live |
| Server notices, scrolling banners, login notices | DB / live |
| Whitelist staff logins during maintenance | DB |
| Reset the world from the `SHARD_INIT` template | DB |
| Backup and restore, and scheduled backups | DB |
| Wipe or archive logs to control database size | DB |
| Live GM actions queued through `_ExeGameServer` | Live |
| Full GM console command surface — spawn, warp, invisible, invincible, weather, item creation, buffs, kill, ban | Live |

## 17. Client build and distribution

This is where a PK2 library is the foundation.

| Capability | Surface |
|---|---|
| Read, edit, add and remove files inside any PK2 | Client archives |
| Repack a PK2 to reclaim space | Client archives |
| Point the client at your server | `divisioninfo.txt`, `gateport.txt` |
| Configure the launcher and version handshake | `SILKCFG`, gateway config |
| Generate a client patch from database changes | DB → client |
| Version and serve patches incrementally | DownloadServer |
| Rebrand: logos, loading screens, UI strings, window title | `Media.pk2` |
| Localise text and add languages | `textdata_*.txt`, `textuisystem.txt` |
| Multi-client patching, so several clients run at once | `sro_client.exe` |
| Remove or replace client-side checks | Client binary |
| Custom launcher with news, patch notes, account links | External |
| Verify client integrity and detect tampering | Client / network |

## 18. Logging and analytics

| Capability | Surface |
|---|---|
| Login and logout history, with IP | `SRO_VT_LOG` |
| Character events: level-ups, deaths, PK | `SRO_VT_SHARDLOG` |
| Item events: creation, deletion, movement, enhancement | Shardlog |
| Trade, stall and drop history between players | Shardlog |
| Monster and unique kill records | Shardlog |
| Silk and gold transaction ledgers | Logs |
| Quest completion and job activity | Logs |
| Fortress war and event outcomes | Logs |
| Live dashboards: concurrency, population by region, economy | Derived |
| Retention, new-player funnel, churn | Derived |
| Suspicious-pattern reports for duping, botting, gold selling | Derived |
| Full audit trail of every admin action taken through the tool | Tool itself |

## 19. Security and anti-cheat

| Capability | Surface |
|---|---|
| Packet filtering in front of the agent server | Network |
| Block known exploits: avatar, invisible/invincible, zerk, char-name injection, gameserver crash | Network |
| Rate-limit and sanity-check packet sequences | Network |
| Detect and act on botting and automation | Network / analytics |
| Detect speed and teleport hacking | Network / live |
| Protect GM accounts with a second password and command gating | Live / DB |
| Restrict GM commands by privilege tier | Live |
| Audit and alert on GM actions | Tool |
| DDoS mitigation and connection limits | Infrastructure |
| HWID and multi-account limits | Network / DB |
| Block trade or stall abuse patterns | Live / analytics |
| Rollback tooling for when something does get through | DB + logs |

## 20. Web and external integration

| Capability | Surface |
|---|---|
| Account registration and email verification | Web + DB |
| Player control panel: password, email, secret question | Web + DB |
| Web item mall and web stall | Web + DB |
| Rankings: level, guild, job, item points, unique kills | Web + DB |
| Vote-for-silk against server listing sites | Web + DB |
| Donation and payment processing into silk | Web + DB |
| Referral and reward-a-friend systems | Web + DB |
| Discord integration: announcements, bots, rich presence | External |
| Public server status and population display | Web |
| Support ticketing and in-game report handling | Web |
| Localised, mobile-friendly staff panel | Web |

## 21. Automation and scheduling

| Capability | Surface |
|---|---|
| Cron-style scheduling of any action | Tool |
| Recurring unique spawns | Scheduled |
| Recurring events and tournaments | Scheduled |
| Fortress war windows | Scheduled |
| Periodic silk and job payouts | Scheduled |
| Scheduled restarts, backups and log rotation | Scheduled |
| Timed rate boosts — happy hours, weekend EXP | Scheduled |
| Seasonal content windows | Scheduled |
| Conditional triggers, e.g. announce when a unique dies | Live hooks |
| Bulk jobs with progress and rollback | Tool |

---

## Cross-cutting problems

These are not features but they determine whether a toolbox is trustworthy, and
they apply to nearly every capability above.

**Database and client must stay in step.** Most content exists twice: a
reference table for the server and a `Media.pk2` text file for the client. Add
an item in SQL alone and the client shows a blank; edit the client alone and the
server rejects it. A toolbox that edits one surface without the other creates
work rather than removing it. Existing tools address this by generating client
patch files directly from database changes.

**Identity and ID allocation.** Several reference tables need their `ID` column
set as a primary key with identity specification before content can be added
cleanly, and IDs must stay consistent across the table graph. Getting this wrong
is one of the most common ways a server ends up subtly broken.

**Referential chains have no enforcement.** Spawns span three tables, shops
span five or more, and nothing stops a half-finished edit. Validation before
writing is worth more than any individual editor.

**Preview, audit and reversibility.** The established pattern is to generate the
T-SQL and offer three modes: copy it out, preview then run, or run directly —
with every action logged. Given that most of this is destructive and operates on
live player data, this matters more than feature count.

**Version and base divergence.** vSRO 1.188 is the common base, but caps,
column sets and available systems vary between server files and between
regional clients. Anything reading a schema should discover it rather than
assume it.

**Live versus offline.** Some changes apply instantly, some need a shard
restart, some need a client patch. Surfacing which is which prevents a great
deal of confusion.

**Scale.** Reference tables run to tens of thousands of rows and log tables grow
without bound. Bulk edits, filtering and log pruning are core, not conveniences.

---

## Sources

- [SRO Developing Docs](https://srodevs.gitbook.io/docs) — file formats, packet
  structures, Lua and CScript, client internals
- [RaGEZONE](https://forum.ragezone.com/) and
  [elitepvpers](https://www.elitepvpers.com/) — the primary development forums
- [vSRO.org](https://www.vsro.org/), [SroArena](https://www.sroarena.com/),
  [Elite Community](https://elitecommunity.org/) — guides and query collections
- [SROUltra](https://sroultra.com/) — an existing control panel; its feature
  list maps much of sections 13, 15 and 21
- [JellyBitz](https://github.com/JellyBitz) — PK2 API, DB-to-client importer,
  launcher, map tools, packet filter, server addon via DLL injection
- [Veykril/pk2](https://github.com/Veykril/pk2) — PK2 read/write reference
- [mahmoudfarouq/pk2](https://github.com/mahmoudfarouq/pk2) — our own PK2 crate,
  and its `docs/` describing the archive format in detail
