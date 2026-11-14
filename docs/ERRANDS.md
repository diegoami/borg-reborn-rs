# Errands

*Commands that give the Borg a job: go home, dive to a depth, explore a
level, climb a ladder of trips. They run over **borg-rs's decision core**,
vendored at borg-rs milestone 12 as `borg-core/` (docs/PROVENANCE.md). The
errand code is `borg-core/src/shell.rs`, plus the hook calls marked
`borg-reborn-rs (#207)` / `(#208)` in the vendored files. The evaluation
behind it is in PR #206 (docs/REDESIGN.md, docs/PILOT.md).*

## Selecting the core

| setting | where | values |
|---|---|---|
| core | `BORG_CORE`, else `borg_reborn_core` in the run's `borg.txt` | `reborn` (default: this repository's Borg, unchanged) or `borg-rs` |
| errand | `BORG_ERRAND`, else `borg_reborn_errand` | a command line, below; `none` |
| fear | `BORG_STOPSCARED`, else `borg_reborn_stopscared` | `off`, `flee`, `1`-`1000` |
| budget | `BORG_ERRAND_BUDGET` | game turns; 0, none |
| outcome | `BORG_ERRAND_OUT` | a file: one JSON line per errand that ends |

Each of these environment settings that is set is written to the run's
`config.txt` (`borg_core=`, `borg_errand=`, ...; `borg_ladders_sha256` for
a ladders file), and `tools/rerun-death.py` sets them again when it reruns a
death (review #227, finding 8).

`harness/borg.txt` is borg-rs's `borg.txt` with its munchkin start by class
and its town scumming off (owner, 2026-11-14) and these keys added; the
zoo copies it. `BORG_CORE_OVERRIDES=off` runs borg-rs's own copy
(`borg-core/borg.txt`), for the parity check.

## Commands

In the game, `^Z /` asks for a line. Each command hands control back (the
core's own stop) when it is done.

| command | what it does | done |
|---|---|---|
| `/town` | home: Word of Recall if the core has one, else the stairs up | in town |
| `/walk-back` | home by the stairs only | in town |
| `/explore-back` | this level and each one above it explored, in turn, then home | in town |
| `/dive N` | the stairs down to DL N, not exploring | on arrival |
| `/explore-level` | the core explores the level it is on | when the level is explored |
| `/dive-exploring N` | each level from here down to DL N explored, in turn | when DL N is |
| `/ladder` | the class's ladder of trips (`borg-core/ladders.txt`) | never (the last trip repeats) |
| `/trips 1*/1,2,1*@3/...` | trips: `/` between trips, `,` between levels, `*` repeats until the core is prepared for the next trip (or `@N`, from character level N) | never |
| `/stopscared off\|flee\|N` | setting: stop and hand control back when the one-turn danger where the Borg stands reaches N% of its hit points; `flee` is 100 | |
| `/explored N\|off` | setting: a level counts as explored at N% of its floor seen (default 90), or, off, only when the core is done with it | |
| `/stall N\|off` | setting: a level is given up after N game turns with nothing new seen (default 10,000) | |
| `/stop` | clear the errand | |

Several commands can go on one line, separated by `;`
(`/stopscared 50; /dive 3; /walk-back`). Settings apply at once. Errands
run in turn, each one starting when the one before it is done. Any other
ending (scared, aborted, the budget) drops the rest.

## Rules

- **On a level of the plan, the core plays as itself.** The errand only
  walks the stairs between levels, after the core's caution.
- **No level is forced.** A ladder skips a level the core is not prepared
  for. When the core climbs back up off the plan inside the dungeon (below
  character level 10 it leaves a level it is not prepared for), a ladder
  trip goes home and a command stops: "the core rose from DL2, not fit for
  DL3". The commands do not skip levels, so `/dive 3` with a level-1
  character stops at the first such rise.
- **Fear overrides the plan.** If the core wants town, the trip ends and
  goes home. If it flees a level, it leaves its own way. A level of the
  plan that it leaves for another dungeon level counts as explored. A trip
  to town is not fear: the core went to shop or restock, and the errand
  takes it back.
- **With no errand set and `/stopscared` off, nothing changes.** Every hook
  returns at once.
