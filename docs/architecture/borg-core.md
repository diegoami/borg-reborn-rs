# The borg-core we ship

`borg-core/` is the decision core that plays by default and the one
borg-zoo measures: borg-rs's crate (a line-by-line Rust translation of the
C Borg), vendored at a pin, with our own declared files and hooks around
it. This page describes that core as it stands in this repository. Its C
ancestor is described in [original-c-borg.md](original-c-borg.md); the two
tracks that work on it (P: calibration, R: the redesign) are in
[reborn-plan.md](reborn-plan.md). The core's own design documents (the
town planner, the trip policy, the class family), written in borg-rs and
kept here since 2026-11-21, are in
[docs/design/](../design/) (§7).

Our own crate `borg-reborn-rs/` is the shell around it: the engine
boundary (`borg-reborn-rs/src/engine.rs`) and the selector between this
core and our own "reborn" core. The shared Ctrl-Z prompt sends PBorg
commands to `do_cmd_pborg` in `borg-reborn-rs/src/engine.rs`. When
`core_is_borg_rs()` selects the vendored core, the shell applies the
configured fear setting and errand, then forwards the command key to
`borg_rs::do_cmd_borg_key()`.

## 1. What it is and where it comes from

### The vendoring

| item | what |
|---|---|
| [`borg-core/PIN`](../../borg-core/PIN) | the borg-rs commit vendored: `8c5d000a58e9ddca1f1954ab9552e505856b313f`, the same as the `borg-rs` pin in [`pins.json`](../../pins.json). The refresh to it (#368, three-way from `1443c2d`, the M13 tag) is recorded in [PROVENANCE.md](../PROVENANCE.md) |
| [`scripts/borg-core-vendor.py`](../../scripts/borg-core-vendor.py) | keeps the copy honest: `delta` (our changes as a patch), `check` (borg-core/ is borg-rs at the pin plus declared files, exit 1 otherwise), `refresh REV` (three-way merge of each file: base the pin, theirs REV, ours borg-core/; writes `PIN`; conflicts left marked) |
| `MAP` (`borg-core-vendor.py:32`) | borg-rs's `borg-rs/` maps to `borg-core/`; its `harness/borg.txt` to `borg-core/borg.txt`; the C Borg's `angband/src/borg/borg-init.c` and `borg.txt` to `borg-core/c-reference/` |
| `OURS` (`borg-core-vendor.py:39`) | files that are ours only and must not exist in borg-rs |
| `HOOKED` (`borg-core-vendor.py:42`) | borg-rs files that carry our changes, each marked in the source `borg-reborn-rs (#N)`, with the issue noted beside the entry |

`check` needs a borg-rs checkout at `vendor/borg-rs` (`scripts/setup.sh`).
At the current tree it reports `borg-core/ against borg-rs 8c5d000a58e9: ok`.
It is run by hand: CI does not call it (`.github/workflows/ci.yml`). A
refresh goes in its own PR, with a parity check against a fresh build of
borg-rs at the new revision (the script's docstring).

The declared lists are the authority on what is ours. Today `OURS` holds
`PIN`, `ladders.txt` and eight modules: `shell.rs`, `params.rs`,
`core_trace.rs`, `attack_preference.rs`, `errands_plan.rs`,
`scary_list.rs`, `borg_lightless.rs`, `borg_town_hunt.rs`. `HOOKED` holds
53 borg-rs files (2026-11-21; the set grows with each task that hooks
the core), from `Cargo.toml` (package rename, a lints table
that allows clippy on borg-rs's code) and `lib.rs` (our module
declarations) to most of the decision modules. Source comments marked
`borg-reborn-rs (#` number 585 across `borg-core/src`.

### Why it is here: the repivot of 2026-11-14

[SPEC.md §2](../SPEC.md) makes everything in this repository independent of
borg-rs, with one exception made on purpose by the owner's amendment of
2026-11-14 (the repivot, option C): borg-rs's decision core is vendored at
a pin as `borg-core/`, selected by `BORG_CORE=borg-rs`, and re-synced at the
borg-rs milestones the owner picks. Our own core still needs nothing from
borg-rs. [GOALS.md](../GOALS.md) records the same amendment in decision D4
("Copy, then own", amended again 2026-11-14): the core is re-vendored on
purpose with `scripts/borg-core-vendor.py`, and track R re-expresses the
vendored decisions in our domain.

### Formatting and checks

borg-core keeps borg-rs's own formatting. CI checks formatting with
`cargo fmt --check -p borg-reborn-rs` only, never the whole workspace, and
no vendored file is ever passed to `rustfmt`: a reformat would turn every
line into a delta against the pin. Clippy runs on the workspace with
`-D warnings`, but `borg-core/Cargo.toml` allows all clippy lints for this
crate (#207). Its unit tests run as `cargo test --release -p borg-core`
(they link without the engine only when unused engine calls are optimised
out). CI also runs `scripts/check-param-citations.py`, which checks that
every parameter's cited source line reads it (#601).

## 2. Module map

`borg-core/src/lib.rs:1` states the rule: one module per C file of
`angband/src/borg/`. Each ported module names its C file in its first
line (`//! Port of borg-caution.c ...`). `borg-core/c-reference/` holds only
two C files for reference, `borg-init.c` and the C Borg's `borg.txt`. The
crate is named `borg_rs` (`borg-core/Cargo.toml`, `[lib] name`). Module
sizes in lines are given where they help.

**Ours** (in `OURS`) is marked **O**. Everything else is borg-rs's,
either a port of a C file or a module borg-rs added itself (its issue is
named). Modules in `HOOKED` carry our marked hooks but are still borg-rs's
code.

| subsystem | modules | C file / origin |
|---|---|---|
| entry and state | `borg.rs` (inkey hook `borg_entry_point`, `generate_keypress`; think loop at `borg.rs:654`), `state.rs` (all Borg state, the former C globals), `borg_commands.rs` (the `^Z` menu) | `borg.c`, the globals of `borg/*.c`, `borg-commands.c` |
| think loop | `borg_think.rs` (`borg_think`, `borg_think.rs:166`), `borg_think_dungeon.rs` (the ordered dungeon list, `borg_think_dungeon.rs:580`), `borg_think_dungeon_util.rs`, `borg_think_store.rs` | `borg-think*.c` |
| danger | `borg_danger.rs` | `borg-danger.c` |
| caution and escape | `borg_caution.rs` (6,888 lines, the largest), `borg_escape.rs`, `borg_recover.rs` | `borg-caution.c`, `borg-escape.c`, `borg-recover.c` |
| fight | `borg_fight_attack.rs`, `borg_fight_defend.rs`, `borg_fight_perm.rs`, `borg_projection.rs` | `borg-fight-*.c`, `borg-projection.c` |
| flows (movement) | `borg_flow.rs`, `borg_flow_dark.rs`, `borg_flow_glyph.rs`, `borg_flow_kill.rs` (also the monster list `borg_kills`), `borg_flow_misc.rs`, `borg_flow_stairs.rs`, `borg_flow_take.rs` | `borg-flow*.c` |
| perception and cave | `borg_update.rs` (knowledge from the frame), `borg_cave.rs`, `borg_cave_light.rs`, `borg_cave_util.rs`, `borg_cave_view.rs`, `borg_light.rs`, `borg_messages.rs`, `borg_messages_react.rs` | `borg-update.c`, `borg-cave*.c`, `borg-light.c`, `borg-messages*.c` |
| items and inventory | `borg_item.rs`, `borg_item_activation.rs`, `borg_item_analyze.rs`, `borg_item_decurse.rs`, `borg_item_enchant.rs`, `borg_item_id.rs`, `borg_item_use.rs`, `borg_item_val.rs`, `borg_item_wear.rs`, `borg_inventory.rs`, `borg_junk.rs`, `borg_trait.rs`, `borg_trait_swap.rs`, `borg_power.rs` | `borg-item*.c`, `borg-inventory.c`, `borg-junk.c`, `borg-trait*.c`, `borg-power.c` |
| magic | `borg_magic.rs`, `borg_magic_play.rs` | `borg-magic.c`, `borg-magic-play.c` |
| preparedness and trips | `borg_prepared.rs` (how deep it is ready to go), `borg_trip.rs` (more reasons to go to town), `borg_reincarnate.rs` | `borg-prepared.c`; borg-rs#60; `borg-reincarnate.c` |
| store and home (C) | `borg_store.rs`, `borg_store_buy.rs`, `borg_store_sell.rs`, `borg_home_notice.rs`, `borg_home_power.rs` | `borg-store*.c`, `borg-home-*.c` |
| town planner (borg-rs) | `borg_buy_strategy.rs`, `borg_town_usefulness.rs`, `borg_town_budget.rs`, `borg_town_stock.rs`, `borg_town_solve.rs`, `borg_town_record.rs`, `borg_town_dispose.rs`, `borg_town_capability.rs`, `borg_town_shadow.rs`, `borg_town_utility.rs`, `borg_town_live.rs`, `borg_town_packing.rs`, `borg_town_traits.rs`, `borg_town_dump.rs`, `borg_object_log.rs` | borg-rs's own: buying strategy (borg-rs#25), the town inventory redesign in phases (borg-rs#36); `borg_town_live` executes the plan with `borg_town_plan = 1` (shipped) |
| formulas, init, log | `borg_formulas.rs`, `borg_formulas_calc.rs` (the FORMULA SECTION of borg.txt), `borg_init.rs` (reads borg.txt), `borg_log.rs`, `borg_util.rs`, `cctype.rs` | `borg-formulas*.c`, `borg-init.c`, `borg-log.c`, `borg-util.c`, `<ctype.h>` |
| engine boundary | `engine.rs` (3,646 lines, the only place with `unsafe` besides `headers`; `lib.rs:17` denies it elsewhere), `headers/` (the engine's raw C declarations), `borg_headers/` (the Borg's own headers, not FFI), `borg_io.rs` (screen reads, messages, key queue) | the engine's headers, `borg-io.c` |
| regression entry | `regress.rs` (engine-free entry points for `harness/regress/`) | borg-rs |
| errands | `shell.rs` **O** (#208), `errands_plan.rs` **O** (#305) | ours |
| calibration | `params.rs` **O** (#219), `attack_preference.rs` **O** (#403, class/family attack ranking) | ours |
| trace | `core_trace.rs` **O** (#222) | ours |
| scary monsters | `scary_list.rs` **O** (#518) | ours |
| added behaviours | `borg_lightless.rs` **O** (#811, recovery in a lightless dungeon), `borg_town_hunt.rs` **O** (#789, town hunts behind `town_hunt`) | ours |

`engine.rs` documents the entry discipline at its top
(`borg-core/src/engine.rs:10`): the engine calls the Borg through four
callbacks (the `inkey_hack` hook, the `EVENT_BELL` and `EVENT_LEAVE_GAME`
handlers, and `do_cmd_borg`), each through `enter`
(`borg-core/src/engine.rs:79`), which hands out the one `&mut BorgState`
and lends it across engine calls that can re-enter the Borg.

## 3. The parameter registry

### Declaring a parameter

All parameters are rows of one `parameters!` macro in
[`borg-core/src/params.rs`](../../borg-core/src/params.rs) (macro at
`params.rs:12`, rows from `params.rs:41`; 2,283 rows today). A row is

```text
name => (default, unit, min, max, "file.rs:LINE", "description" [, [9 class defaults]])
```

and generates both the `Param` enum and the `SPECS` table, so a value
cannot drift from its name. `Spec` (`params.rs:29`) holds name, default,
unit, inclusive range, source (the first live line that reads it, checked
by `scripts/check-param-citations.py`), description and optional compiled
per-class defaults, in `class.txt` order (`CLASSES`, `params.rs:2344`). The
description carries the evidence: the issue, often the owner's words, and
what 0 or the default keeps ("0 retains today's play"). Code reads a
parameter with `params::value(Param::name)` (`params.rs:2408`), usually
imported as `p` (for example `borg_caution.rs:61`); `resolved`
(`params.rs:2398`) also returns the origin. About 30 modules read
parameters.

### The human record

[CALIBRATION.md](../CALIBRATION.md) (about 5,100 lines) is the record of
the registry. It opens with the P1/P2 totals, then one section per change,
newest first (`## Walk out of a slower crowd (#690)` and so on). Each
section says what the parameters do, the registry count right after that
change ("The registry now holds N"), the evidence, and a table of
parameter, default, unit, range and source. Older shared sections follow:
"Configuration and records" (precedence, validation, `params-core.json`),
"Units and arithmetic", "Constants retained, and why", the parameter
catalogues of P1.1–P2.4 and later catalogues (#328, #403, #768 and more).

### How a setting reaches the code

Values are resolved once per process (`params::initialize`,
`params.rs:2624`), called from `borg_init` before any RNG or decision
(`borg-core/src/borg_init.rs:996`), and are immutable for the process,
reincarnations included. Which class's value applies follows the character.

| layer | form | where |
|---|---|---|
| compiled default | the row's `default`, or its class default | `params.rs`; neutral: it keeps the vendored policy (#479) |
| shipped value | `borg_param_<name> = N` | [`harness/borg.txt`](../../harness/borg.txt) (353 such lines), copied as the run's `user/borg.txt` |
| class value | `borg_param_<name>@<class> = N`, or `<name>__<class>` (#696: the characters a zoo profile can write) | written by hand in `harness/borg_<class>.txt`, folded into borg.txt's `CLASS FILES` block (`harness/borg.txt:1374`) by `scripts/borg-class-files.py fold` (`check` fails if the block is stale) |
| legacy alias | `attack_preference_<class>_<family>_permille`, `attack_arrow_point_blank_divisor_<class>`, `town_heal_*_<class>` | accepted as the class value (`legacy_alias`, `params.rs:2447`); a value at the family's old "general" sentinel sets nothing |
| environment | `BORG_PARAM_<NAME>`, and `BORG_PARAM_<NAME>__<CLASS>` for one class | `split_env_name` (`params.rs:2494`); upper case only, a lower-case name is unknown |
| zoo profile | rewrites a borg.txt line in place | [ZOO-INTERFACE.md](../ZOO-INTERFACE.md); `harness/run.sh` offers the same with `BORG_TXT_SET="name=value;..."` for existing keys |

Precedence inside `parse` (`params.rs:2519`): the environment, then the
file, then the compiled default, separately for the common value and for
each class. A class value from either source wins over the common value;
compiled class defaults apply only when the common value is the default.
Invalid integers, values out of range, unknown names or classes and
duplicates stop the run with exit status 2; nothing is clamped. Retired
depth-cap names (#768) are skipped with a warning. Lookup (`pick`,
`params.rs:2390`) takes the current class's value if one exists, else the
common one. The class is set by `set_class` (`params.rs:2381`) when
parameters load and again in `borg_prepare_race_class_info`
(`borg_init.rs:971`) at birth and reincarnation.

The same class rule covers the C Borg's own `borg_*` settings (#520's
second half): `borg_init.rs` reads `borg_<name>@<class>` or
`borg_<name>__<class>` lines apart (`class_setting_line`,
`borg_init.rs:438` and its definition below), records the common values
(`record_class_commons`) and applies the class's lines in
`apply_class_settings` (`borg_init.rs:831`), at file load
(`borg_init.rs:755`) and at birth (`borg_init.rs:975`). Respawn settings
and the shell's keys have no class value.

**Compiled defaults versus shipped values (#479).** The compiled core
keeps neutral defaults that reproduce the vendored policy. The behaviour we
ship comes from `harness/borg.txt` (its header says so). A zoo profile that
omits a key therefore gets the shipped value, not the compiled default, and
`BORG_CORE_OVERRIDES=off` (§6) plays the compiled defaults with borg-rs's
own file.

### Records of a run

With `BORG_RUN_DIR` set, initialization writes `params-core.json`
(every parameter: spec, effective value, origin `default`, `borg.txt` or
`environment`, and class values; a failed write exits 2). After birth,
`write_settings_receipt` (`params.rs:2683`) writes
`settings-resolved.json` (schema `borg-settings-resolved@1`, #798): the
active class and every parameter whose value differs from its compiled
class default. Death dossiers keep `params-core.json` and its hash.

## 4. The decision trace

[CORE-TRACE.md](../CORE-TRACE.md) is the contract;
`borg-core/src/core_trace.rs` the code. Every completed core think writes
one JSON line to `BORG_RUN_DIR/trace-core.jsonl` (`core_trace.rs:240`):
`borg_think` (`borg_think.rs:166`) wraps the think in `core_trace::begin`
and `core_trace::finish`. `BORG_CORE_TRACE=off` (or `0`, `false`) turns it
off (`core_trace.rs:235`); without a run directory (or a frontend summary
path) it is off. It is separate from the reborn core's `trace.jsonl`.

A line, schema `borg-reborn-rs/core-trace@2` (`core_trace.rs:306`), holds:
the think sequence; engine turn and the core's clock; context (town, store
or dungeon, depth, HP and SP); the danger at the Borg's position and its
avoidance; `check`, the root priority check that returned (for example
`borg_caution`, `borg_recover`, `borg_attack`) with `check_source`; the
goals (leaving, fleeing, fleeing to town, rising); the errand; the queued
keys and whether the think acted; and `attacks`, every attack ranking of
the think with its candidates and winner ([ATTACK-PREFERENCES.md](../ATTACK-PREFERENCES.md)).
Later additions record live preparedness checks (#795,
`core_trace::record_prepared`, `core_trace.rs:105`).

How a check is named: each successful return in `borg_think.rs` and
`borg_think_dungeon.rs` goes through the `choice!` macro
(`core_trace.rs:271`; 124 uses), which records the name and its
`file!():line!()` and returns the original `true`; store and errand
returns call `core_trace::selected` (`borg_think.rs:240`, `:279`).

**It never changes play.** No trace value feeds back into a decision;
the extra danger evaluation is read-only and draws no RNG; full-suite
parity guards that. A write failure exits 2. A death dossier keeps the last
200 lines as `trace-core-tail.jsonl`. CI asserts the schema and that no
row is `unclassified`.

## 5. Our shell and errands around the core

### Errands

[ERRANDS.md](../ERRANDS.md) is the user's guide;
`borg-core/src/shell.rs` the code (#208). An errand gives the core a job;
on a level of the plan the core plays as itself, no level is forced past
`borg_prepared`, fear overrides the plan, and with no errand and
`/stopscared` off every hook returns at once, so play is borg-rs's byte for
byte (`shell.rs`, module comment).

| setting | environment | borg.txt key |
|---|---|---|
| errand | `BORG_ERRAND` | `borg_reborn_errand` (shipped `none`) |
| fear | `BORG_STOPSCARED` | `borg_reborn_stopscared` (shipped `off`) |
| budget | `BORG_ERRAND_BUDGET` (game turns) | |
| outcome | `BORG_ERRAND_OUT` (one JSON line per errand that ends, `shell.rs:1280`) | |
| ladders | `BORG_LADDERS` (else `borg-core/ladders.txt`, `shell.rs:262`) | |

The shell reads these on each `^Z` entry
(`borg-reborn-rs/src/borg.rs:343`, `:379`) and validates them strictly:
a bad value exits 2.

### The `^Z /` command line

Our one entry in the C Borg's `^Z` menu is `/`
(`borg-core/src/borg_commands.rs:109`), which prompts for a line
(`borg_cmd_errand`, `shell.rs:551`). The other `^Z` keys are the C Borg's.
`Command::parse` (`shell.rs:136`) accepts, with the slash optional:

| command | kind |
|---|---|
| `/town`, `/shopping`, `/walk-back`, `/explore-back`, `/explore-level` | errand |
| `/dive N`, `/dive-exploring N` | errand, N ≥ 1 |
| `/ladder`, `/trips 1*/1,2,1*@3/...` | errand that never ends |
| `/stopscared off\|flee\|N` (1–1000; `true`/`on`/`yes` = `flee` = 100) | setting |
| `/explored N\|off` (1–100) | setting |
| `/stall N\|off` | setting |
| `/stop` (or `none`) | clears the errand |

Commands on one line are separated by `;` (`parse_line`, `shell.rs:521`).

The shell's hooks in the vendored core, all marked `borg-reborn-rs
(#208)`: `before_dungeon_think` (`borg_think.rs:278`), `steer`
(`borg_think_dungeon.rs:819`), `on_leave_level` and `on_leave_choice`
(`borg_think_dungeon_util.rs:733`, `:1019`), `shopping_departure`
(`borg_caution.rs:2764`).

### Plans

An errands plan ([GAME-PLAN-LANGUAGE.md](../GAME-PLAN-LANGUAGE.md),
`errands_plan.rs`, #305) is a plain-text file of class sections and
character-level ranges, one `max-depth N` per phase (the core plays as
itself under a depth cap; `loop-levels` was removed 2026-11-16). It is
selected by `borg_plan = FILE` in the run's borg.txt (resolved under the
user directory) or `BORG_ERRANDS_PLAN`, read once and validated before play
(`errands_plan::initialize`, `borg_init.rs:995`). Two plans ship in
[`harness/plans/`](../../harness/plans/README.md): `errands_plan.txt` and
`errands_plan_ambitious.txt`. A plan with the reborn core exits 2
(`borg-reborn-rs/src/borg.rs:319`).

### The scary list

The C Borg marks a level scary while certain named monsters are known and
the character is at or below a level. `scary_list.rs` (#518) reads that
list from a file instead: [`harness/scary_monsters.txt`](../../harness/scary_monsters.txt),
selected by `borg_scary_list = scary_monsters.txt` (`harness/borg.txt:1197`)
and used with `borg_param_caution_scary_named_monsters = 1`
(`harness/borg.txt:1190`). A line is `monster | tough-fragile level | class
exceptions | flags`, scaled by per-class fragility; later lines overlay
earlier ones. `borg_scary_list = none` or no key keeps the compiled C list.
An unknown monster or class stops the run (`scary_list::selected`,
`scary_list.rs:276`, loaded in `borg_init`, `borg_init.rs:998`).

## 6. How a run is wired

`borg-core/src/borg_init.rs:693` (`borg_init_txt_file`) reads `borg.txt`
from the game's user directory (`engine::path_build_user`,
`borg_init.rs:709`). If the file is missing it warns and writes one with
the defaults. The shell and the registry read the same file through
`shell::borg_txt_contents` (`shell.rs:241`); its missing-file policy is
"settings absent", while any read error or invalid UTF-8 exits 2
(ERRANDS.md). `^Z $` reloads the C settings in game; the parameters
(`params.rs`, a `OnceLock`) and the core selection stay as the process
first read them.

[`harness/run.sh`](../../harness/run.sh) builds the run's `user/`:

1. **borg.txt.** It copies `harness/borg.txt` to `OUTDIR/user/borg.txt`,
   setting `borg_respawn_race`, `borg_respawn_class` and
   `borg_cheat_death = FALSE` (`run.sh:233`–`247`). With
   `BORG_CORE=reborn` it rewrites `borg_reborn_core = reborn` so the file
   names the core that played (`run.sh:253`). `BORG_TXT_SET` rewrites
   existing keys. A borg.txt that selects a plan itself is refused
   (`run.sh:267`).
2. **The scary list.** `BORG_SCARY_LIST`, else the list borg.txt names
   (from `harness/`), is copied beside it, and the `borg_scary_list` line is
   rewritten with a comment naming its SHA-256 (`run.sh:278`–`293`).
3. **A plan.** `BORG_ERRANDS_PLAN` is copied as `user/errands_plan.txt`
   and selected with `borg_plan = errands_plan.txt` after a SHA-256 comment
   (`run.sh:327`–`335`); it needs the borg-rs core. A world (`borg_world`,
   #704) is handled the same way.
4. **Identity.** `config.txt` records `borg_txt_sha256`, so plan, list and
   world bytes are part of the run's identity, the `BORG_CORE`,
   `BORG_ERRAND` and similar variables that are set (`run.sh:366`), and,
   with no `BORG_CORE`, the core the file names (`borg_core=`).

**Which core plays.** `BORG_CORE`, else `borg_reborn_core` in the run's
borg.txt; `parse_core_selector` (`borg-reborn-rs/src/borg.rs:328`) accepts
exactly `borg-rs`/`1` and `reborn`/`0`, and anything else exits 2. With
neither set, the compiled fallback is the reborn core. The shipped
`harness/borg.txt:37` says `borg_reborn_core = borg-rs` (#479), so with
`BORG_CORE` unset a harness run and a zoo cell play this core (#606). The
choice is read once per process.

**`BORG_CORE_OVERRIDES=off`** gives the core borg-rs's pristine
`borg-core/borg.txt` instead (`run.sh:240`): no `borg_param_*` or
`borg_reborn_*` keys, so compiled defaults and no errand. That file names
no core, so `run.sh:237` exports `BORG_CORE=borg-rs` for it. It is the
parity check against borg-rs's own build at the pin.

The default core's regression reference is
`harness/reference/smoke-borg-rs`. A change to a shipped value in
`harness/borg.txt` is committed, then `scripts/rerecord-smoke-borg-rs.sh`
re-records the reference ([AGENTS.md](../../AGENTS.md)).

## 7. Design documents and further reading

The core is borg-rs's, and so is most of its design. Its design documents
moved here from borg-rs on 2026-11-21 and are ours now, in
[docs/design/](../design/):

| Subsystem (§2) | Read |
| --- | --- |
| town planner (`borg_town_*.rs`, `borg_buy_strategy.rs`) | [TOWN-INVENTORY.md](../design/TOWN-INVENTORY.md) (architecture), [TOWN-UTILITY.md](../design/TOWN-UTILITY.md) (the utility model), [TOWN-ITEM-CATALOGUE.md](../design/TOWN-ITEM-CATALOGUE.md), [TOWN-LIVE.md](../design/TOWN-LIVE.md) (the live executor), [TOWN-SESSION.md](../design/TOWN-SESSION.md) |
| preparedness and trips (`borg_prepared.rs`, `borg_trip.rs`) | [TRIP-POLICY.md](../design/TRIP-POLICY.md), [JOINT-POLICY-TRIALS.md](../design/JOINT-POLICY-TRIALS.md) |
| shops (`borg_store*.rs`, `borg_think_store.rs`) | [BORG-SHOP-SHORTCUTS.md](../design/BORG-SHOP-SHORTCUTS.md) |
| what borg-rs kept or fixed of the C Borg's bugs | [c-bugs.md](../design/c-bugs.md), with the C defect catalogue [c-borg/c-bugs/](../c-borg/c-bugs/README.md) |
| the family of versions | [FAMILY.md](../design/FAMILY.md) |

What we added to it since the repivot is in the handover notes of each
task ([docs/handover/](../handover/README.md)) and, for its settings, in
[CALIBRATION.md](../CALIBRATION.md); single decisions are mapped in
[PHASE-DOOR.md](../PHASE-DOOR.md) and [ATTACK-PREFERENCES.md](../ATTACK-PREFERENCES.md).
