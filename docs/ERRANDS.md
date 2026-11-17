# Errands

*Commands that give the Borg a job: go home, dive to a depth, explore a
level, climb a ladder of trips. They run over **borg-rs's decision core**,
vendored at the M13 town/trip integration tag as `borg-core/` (docs/PROVENANCE.md). The
errand code is `borg-core/src/shell.rs`, plus the hook calls marked
`borg-reborn-rs (#207)` / `(#208)` in the vendored files. The original
evaluation is preserved in [closed PR #206](https://github.com/diegoami/borg-reborn-rs/pull/206).*

## Selecting the core

| setting | where | values |
|---|---|---|
| core | `BORG_CORE`, else `borg_reborn_core` in the run's `borg.txt` | `borg-rs` (the shipped default since #479; the reborn core is not ready) or `reborn` (this repository's Borg; `harness/run.sh` still runs it when `BORG_CORE` is unset) |
| errand | `BORG_ERRAND`, else `borg_reborn_errand` | a command line, below; `none` |
| fear | `BORG_STOPSCARED`, else `borg_reborn_stopscared` | `off`, `flee`, `1`-`1000` |
| budget | `BORG_ERRAND_BUDGET` | nonnegative game turns; absent, `0` or `none` means unbounded |
| outcome | `BORG_ERRAND_OUT` | a file: one JSON line per errand that ends |

Configured errands are validated before the vendored core starts. An absent
setting, an empty string, or exactly `none` disables errands. Unknown or
malformed command lines, including an invalid command after a valid prefix,
stop the process with exit status 2 and the setting's source/value on
stderr. `BORG_ERRAND` takes precedence over `borg_reborn_errand`, including
`none` overriding a file command. Valid command syntax and interactive
prompt handling are unchanged. The original reborn core ignores this
vendored-core setting.

The configured fear threshold is also strict: invalid values exit 2 with
source/value diagnostics. `BORG_STOPSCARED` overrides the file setting;
absence, empty and exact `none` retain the core default and suppress a file
override. Explicit `off` disables the threshold. Existing boolean aliases
(`true`/`on`/`yes` for `flee`, `false`/`no` for `off`) and thresholds 1–1000
retain their behavior. The original core ignores this setting. The
stopscared command parser trims its argument before parsing; configured
budget integers reject surrounding whitespace. Empty fear/errand settings
retain their documented default/disabled aliases, while an empty budget is
an error. Exact `none` keeps its documented meaning for each setting.

With a configured vendored-core command line, `BORG_ERRAND_BUDGET` must be
an integer from 0 through 9223372036854775807, or exactly `none`. Missing,
`0` and `none` retain the unbounded meaning. Negative, empty, non-integer,
overflowing or non-UTF-8 values exit 2 with source/value diagnostics instead
of removing the bound. The budget is ignored when no vendored-core command
line is configured, or with the original core.

A missing `borg.txt` means file settings are absent. Other read errors,
including invalid UTF-8 anywhere in the file, stop configuration with exit
status 2 and a path/error diagnostic. Environment overrides retain priority;
a file is read only when a setting or the parameter loader consults it. Core
parameters use the same read policy.

Fear and errand settings are read on command entry, including re-entry
(`^Z`); core selection and the parameter snapshot remain process-cached. In
a vendored-core session, a newly invalid setting or unreadable file
encountered by that reader terminates the process without an automatic save.
Edit configuration before starting a fresh process, and preserve an
interactive game's save before changing it. The reader's valid-to-invalid
re-entry path is covered in a subprocess test; live interactive driving and
automatic save recovery are not claimed.

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

The level-loop errand (`/loop-levels A B`) was removed (owner,
2026-11-16): it forced a route with planned town visits, and its town
restarts caused the #421 stair livelock and slowed casters
(borg-cemetery#19). Plans now cap depth instead (`max-depth N`,
docs/GAME-PLAN-LANGUAGE.md); `/loop-levels` is an unknown command.

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
  prepared for DL3"). A
  ladder also skips a planned level the core is not
  prepared for. When the core climbs back up off the plan inside the
  dungeon by itself (below character level 10 it leaves a level it is not
  prepared for), a ladder trip goes home and a command stops ("the core
  rose from DL2, not fit for DL3"). The commands do not skip levels.
- **Fear overrides the plan.** If the core flees to town, or climbs out of
  a depth it is not prepared for, the trip ends and goes home, or a
  finite command stops. If it flees a
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
