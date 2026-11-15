# Errands

*Commands that give the Borg a job: go home, dive to a depth, explore a
level, climb a ladder of trips. They run over **borg-rs's decision core**,
vendored at borg-rs milestone 12 as `borg-core/` (docs/PROVENANCE.md). The
errand code is `borg-core/src/shell.rs`, plus the hook calls marked
`borg-reborn-rs (#207)` / `(#208)` in the vendored files. The original
evaluation is preserved in [closed PR #206](https://github.com/diegoami/borg-reborn-rs/pull/206).*

## Selecting the core

| setting | where | values |
|---|---|---|
| core | `BORG_CORE`, else `borg_reborn_core` in the run's `borg.txt` | `reborn` (default: this repository's Borg, unchanged) or `borg-rs` |
| errand | `BORG_ERRAND`, else `borg_reborn_errand` | a command line, below; `none` |
| fear | `BORG_STOPSCARED`, else `borg_reborn_stopscared` | `off`, `flee`, `1`-`1000` |
| budget | `BORG_ERRAND_BUDGET` | game turns; 0, none |
| outcome | `BORG_ERRAND_OUT` | a file: one JSON line per errand that ends |

Configured errands are validated before the vendored core starts. An absent
setting, an empty string, or exactly `none` disables errands. Unknown or malformed command lines,
including an invalid command after a valid prefix, stop the process with
exit status 2 and the setting's source/value on stderr. `BORG_ERRAND` takes
precedence over `borg_reborn_errand`, including `none` overriding a file
command. Valid command syntax and interactive prompt handling are unchanged.
The original reborn core ignores this vendored-core setting.

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
| `/shopping` | in town, run the core's buying, selling and home-inventory round | when shopping is finished, before leaving town |
| `/loop-levels A B` | explore every level from A to B, then back, repeatedly; 0 is town | never; town detours restart the route |
| `/walk-back` | home by the stairs only | in town |
| `/explore-back` | this level and each one above it explored, in turn, then home | in town |
| `/dive N` | the stairs down to DL N, not exploring | on arrival |
| `/explore-level` | the core explores the level it is on | when the level is explored |
| `/dive-exploring N` | each level from here down to DL N explored, in turn | when DL N is |
| `/ladder` | the class's ladder of trips (`borg-core/ladders.txt`) | never (the last trip repeats) |
| `/trips 1*/1,2,1*@3/...` | trips: `/` between trips, `,` between levels, `*` repeats until the core is prepared for the next trip (or `@N`, from character level N) | never |
| `/stopscared off\|flee\|N` | setting: stop and hand control back when the one-turn danger where the Borg stands reaches N% of its hit points; `flee` is 100 | |
| `/explored N\|off` | setting: a level counts as explored at N% of its floor seen (default 90), or, off, only when the core is done with it | |
| `/stall N\|off` | setting: a level is given up after N game turns with nothing new seen (default 10,000). It also bounds how long the errand tries to move on from one level, at least 10,000 game turns: then a command stops ("no way on") and a ladder lets the core play on until the level changes | |
| `/stop` | clear the errand | |

Several commands can go on one line, separated by `;`
(`/stopscared 50; /dive 3; /walk-back`). Settings apply at once. Errands
run in turn, each one starting when the one before it is done. Any other
ending (scared, aborted, the budget) drops the rest.

`/shopping` uses the core's existing choices about what is worth buying,
selling, storing at home or taking from home. It does not force a purchase
or a visit to a store with nothing useful to do. The game's selling option
still applies (the default is no selling for gold). It stays in town and
hands control back outside the store UI when the core would normally leave
after shopping. If caution needs to descend, it aborts and hands control back.
Calling it in the dungeon aborts; use `/town; /shopping` to return first.
`BORG_ERRAND_BUDGET` and `/stall` bound the round; no completion ranking of
the errands has been measured.

`/loop-levels 3 5` explores `3 → 4 → 5 → 4 → 3 → …`; `/loop-levels 5 3`
starts at DL5 and goes the other way first. Supply two distinct depths in
the engine's range, 0–127. `/loop-levels 0 3` includes town and the core's
normal shopping/restocking at that endpoint, then repeats the dungeon trip.

The core's own decision to go to town overrides a loop, including fear,
restocking and an unprepared descent. The loop waits for arrival in town
and restarts from its first endpoint after the core's town work. It does
not skip an unprepared level or force a descent. A missing route lets the
core try its own exploration/movement after the existing stall bound.
Return travel can use the core's Recall; departure from town uses its usual
Recall supply/readiness policy, otherwise stairs. Recall goes to the game's
recall depth, not an arbitrary loop endpoint: after landing, the loop walks
to the next planned level and explores it there. Pending Recall is allowed
to finish. `/stopscared`, the errand budget and `/stop` still apply. An
endless loop does not start following queued errands; budget/scared endings
and `/stop` clear the remaining queue.

## Rules

- **On a level of the plan, the core plays as itself.** The level is
  explored when the core is bored with it and free to go either way: it
  has weighed its own reasons to climb or go home (readiness for this
  depth, restocking, selling, fixed level or experience) and none holds;
  or `/explored` or `/stall` says so. Between levels the errand walks the
  stairs, after the core's caution.
- **No level is forced.** The errand never takes the stairs down to a
  level the core is not prepared for (`borg_prepared` for the level below):
  a ladder trip goes home instead, a command stops ("the core is not
  prepared for DL3"); a level loop goes home and retries after town. A
  ladder also skips a planned level the core is not
  prepared for. When the core climbs back up off the plan inside the
  dungeon by itself (below character level 10 it leaves a level it is not
  prepared for), a ladder trip goes home and a command stops ("the core
  rose from DL2, not fit for DL3"). The commands do not skip levels.
- **Fear overrides the plan.** If the core flees to town, or climbs out of
  a depth it is not prepared for, the trip ends and goes home, or a
  finite command stops; a level loop restarts after town. If it flees a
  level, it leaves its own way, and a level
  of the plan it leaves for another dungeon level counts as explored.
- **A trip to town of the core's own is not fear.** When it goes home to
  restock, sell, fix its level or experience, or because it misses home,
  it goes its own way and the plan waits: after town the errand takes it
  back to the level it was on.
- **With no errand set and `/stopscared` off, nothing changes.** Every hook
  returns at once.

## Calibration (track P)

The vendored core's caution thresholds can be set independently of errands.
See [CALIBRATION.md](CALIBRATION.md) for the registry, precedence, bounds
and run/dossier output. Defaults retain the existing policy.
