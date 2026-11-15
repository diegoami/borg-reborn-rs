# Simple profiles and errand plans inside borg.txt

Status: revised design proposal for PR #308, 2026-11-15. The owner requested
an embedded YAML block in `borg.txt`, not a separate TOML/JSON profile system
or a general programming language. The block parser and plan runner are not
implemented by this document. Current commands remain in [ERRANDS.md](ERRANDS.md).

## One configuration file

Keep ordinary settings exactly where they are. Add one here-document-style
setting whose contents are structured YAML:

```text
# Existing settings keep their existing meaning.
borg_reborn_core = borg-rs
borg_param_caution_flee_danger_hp_multiplier = 2

borg_reborn_plan = <<BORG_PLAN
version: 1
initial_profile: cautious
town: resume
profiles:
  normal:
    parameters: {}
  cautious:
    parameters:
      caution_flee_danger_hp_multiplier: 1
plans:
  start:
    - repeat:
        until: {level_at_least: 5}
        steps:
          - town
          - {profile: cautious}
          - shopping
          - {errand: loop-levels, args: [0, 2], turns: 10000}
    - town
    - {profile: normal}
    - {errand: loop-levels, args: [3, 5]}
BORG_PLAN
```

`EOF` could be used instead of `BORG_PLAN`; it is a chosen delimiter, not
part of YAML. The sample parameter values illustrate configuration, not a
recommendation or a claim of improved play. Read parameter names/bounds in
[CALIBRATION.md](CALIBRATION.md).

The initial named plan is `start`. Without `initial_profile`, begin with
the existing flat settings; `profiles` is optional until a profile is used.
An explicit initial profile must exist in the block. A profile is a named set of parameter
settings in `profiles`; a plan is a named sequence in `plans`. Selecting
`profile normal` changes settings, not the running plan. Explicit selection
of another named plan is a proposed `/plan NAME` command. No network lookup,
external plan files, imports, mutable variables, general expressions or custom
functions are needed. Unique tactics and LLM advice remain later proposals.

## Where to read and edit it

The active configuration is the run's `ANGBAND_DIR_USER/borg.txt`:

- Normal watched game: `.angband-reborn/user/borg.txt`.
- Guided/private game: `SESSION/user/borg.txt`, using the directory recorded
  in that session's manifest. Do not edit another run's file by accident.
- Headless harness: `OUT/user/borg.txt`, generated from `harness/borg.txt`.

Reviewed shared examples can live in `harness/profiles/borg-rs/*.borg.txt` as
complete Borg configurations or clearly labelled copyable fragments. The
runtime still reads only its actual `borg.txt`; it does not search that
folder, silently merge fragments or resolve profile names to other files.
Copy the desired block/settings into the run configuration when preparing it.
There is one authoritative block, not a default ladder plus several hidden
profile files. The existing original-core `reborn-default.txt`/`BORG_PROFILE`
and zoo catalogue JSON interfaces retain their separate meanings.

Today the vendored core loads flat `borg_param_<name> = INTEGER` settings
once at initialization. It does not yet recognise this embedded plan syntax
or reload parameters while playing. Do not paste the example into an old
binary and assume a plan is running. This proposal specifies the required
parser/validation change before the syntax becomes supported.

## Block parsing and validation

The proposed header is `borg_reborn_plan = <<DELIMITER`. After recognizing
it, a dedicated block reader consumes literal lines until a line containing
exactly `DELIMITER`, starting in column zero. Delimiters are nonempty ASCII
identifiers without whitespace. Normalize CRLF line endings for comparison;
do not trim content indentation or interpolate environment/shell variables.
A delimiter-like string inside an indented YAML scalar does not end the block.
Choose a different delimiter if a root-level content line would collide.

The ordinary flat-setting reader must **skip the entire block**, including
both markers. YAML content must never be interpreted as flat Borg settings,
formula rules or errand command lines. The reader must also be used when
extracting core/errand settings, so a similarly named key inside YAML cannot
change backend selection. Preserve existing borg.txt sections and comments.
Do not bolt this onto a parser that still scans every embedded line separately.

Allow one block per file. Reject duplicate blocks, missing terminators,
empty/malformed content, duplicate YAML keys, unknown schema versions/fields,
invalid names, excessive sizes and unsupported steps. Use a JSON-compatible
YAML subset: mappings, sequences and plain/quoted scalars only; no custom
tags, object construction, anchors, aliases or merge keys. Do not coerce a
boolean/string into an integer parameter. JSON objects could be accepted
through the same schema later, but v1's documented format is this YAML block.
No second free-form script language is stored inside YAML strings.

Validate the complete block, all referenced profiles/plans and the effective
backend before beginning play. An explicitly requested invalid plan fails
loudly, rather than silently starting ordinary Borg play. With no block,
existing flat settings and slash commands retain their current behaviour.
A nonempty configured `borg_reborn_errand` conflicts with an automatic plan:
reject the combination instead of inventing a precedence. Empty/`none`
means disabled. Starting or replacing a plan interactively is explicit.

## The small plan vocabulary

Use YAML data, with these step forms only:

| Form | Meaning |
| --- | --- |
| `town`, `shopping`, `explore-level`, etc. | One existing primitive errand with no arguments |
| `{errand: dive, args: [3]}` | A primitive errand with checked arguments |
| `{errand: loop-levels, args: [0, 2], turns: 10000}` | A bounded chunk, measured in game turns |
| `{profile: cautious}` | Select settings from the same block's named profile |
| `repeat: {until: {level_at_least: 5}, steps: [...]}` | Check before each iteration, repeat the sequence |
| `repeat: {forever: true, steps: [...]}` | Repeat a yielding sequence indefinitely |

The fixed `until` alternatives are `level_at_least`, `max_depth_at_least`
and `ready_for`, each with one checked integer. Exactly one guard or
`forever: true` is allowed. No compound expressions or nested repeats in v1.
The top-level `town` policy is `resume` (default) or `stop`.

Guards are checked **between errands**, not continuously during a child.
An endless errand inside a guarded repeat must have a finite `turns` budget;
otherwise the guard could never be checked again. A repeat must contain a
yielding errand. Reject empty/profile-only loops and steps after an endless
final errand/repeat. The host bounds control work per game decision as well.
A deliberate chunk budget is normal completion for this plan runner;
`scared`, unexpected abort, impossible action and errors hand back to the
player. Existing slash queues still clear on budget/abort: use an explicit
adapter instead of changing those commands or concatenating strings.

## Profiles and town recovery

The base is existing flat borg.txt settings over registry defaults. Selecting
a profile overlays its parameter table on that base, then optional class
parameter overrides. It replaces the previous profile overlay rather than
accumulating every previously selected profile's values. Thus `normal` with
an empty table returns to the flat Borg configuration, not necessarily to
compiled defaults. Explicit `BORG_PARAM_*` environment overrides are run
locks; reject a conflicting profile value visibly rather than claiming it
was applied. Backend, engine constants and birth options stay outside profiles.

For v1, switch profiles only **in town between completed errands**, with no
pending prompt or half-executed command. A profile step reached in the dungeon
stops with a request to return first; include an explicit `town` step, as in
the example. Switching during a fight is separate future work. Validate every
key, registry bound, allowed mutability and combination before committing.
Apply the whole overlay and required cache refreshes together; failure leaves
the old profile intact. Current startup-loaded parameters need an explicit
live-update audit even for town switching. Reject startup-only keys in a live
switch, instead of pretending a YAML parser makes every parameter mutable.

Core emergency/town intent overrides the plan. `town: resume` preserves the
plan position, completes recovery, then retries the interrupted errand with
fresh readiness/supply checks; `stop` hands back. Do not re-enter one town
request every think. `loop-levels` already owns its town detour and first-endpoint
restart: observe that child state instead of scheduling a duplicate trip.
Manual stop cancels the plan and cannot be caught by a repeat.

Read and freeze the relevant configuration bytes at startup. File edits are
not automatically applied mid-fight. A proposed explicit reload at an idle town
boundary validates the complete file/block first, then restarts the selected
named plan and records old/new configuration hashes. Pause/resume without
reload keeps the cursor and the frozen settings. An Angband save alone does
not restore Borg/plan memory: use qualified joint checkpoints/replay or label
a fresh-state restart honestly.

## Put ladder and class policy in the block

A profile can own named plans and optional single-level class overrides:

```yaml
profiles:
  normal:
    parameters: {}
    plans:
      ladder:
        - {errand: loop-levels, args: [0, 2]}
    classes:
      warrior:
        plans:
          ladder:
            - {errand: loop-levels, args: [0, 3]}
```

This shows storage/lookup, not an equivalent translation of today's ladder.
`/ladder` resolves the matching class's `ladder` plan, then the selected
profile's common plan. Top-level plans are whole-game entry points; profile
plans are named routines selected explicitly, not automatically on switching.
Unknown names/classes fail validation; absent class entries use the common plan.

Migration first relocates existing ladder schedules/gates into profile-owned
data behind a compatibility adapter. Preserve class-specific readiness/skip
rules, level gates, last-trip repetition and town behaviour before retiring
`ladders.txt` or the old executor. If the reduced format cannot represent a
legacy case, retain and document the deprecated adapter; do not call an
approximation equivalent or grow another general language to remove it.
`/trips` may remain a deprecated compatibility input, not new engine policy.

Later add reviewed named unique tactics and preparation kits to profiles,
using accepted item/spell/combat APIs and separate tests. No free-form tactical
language is part of v1. [PR #309](https://github.com/diegoami/borg-reborn-rs/pull/309)
may later propose a named profile/plan/tactic or supported bounded action,
without writing arbitrary code or modifying this configuration file.

## Delivery and checks

This is design only, not an implementation milestone or a SPEC/RULES amendment.
Deliver the literal block reader and schema checks first, then sequential
errands/repeats and town resume, then audited town-only profile changes and
compatibility-preserving ladder migration. R control APIs and the accepted P
registry remain dependencies; inventory work depends on #293/R4, and later
unique tactics on accepted R2/R3/R4 capabilities. Track B remains undefined
in current main: clarify it before scheduling, rather than inventing a track.

Acceptance checks cover flat-file compatibility and block isolation, CRLF and
indented delimiters, truncation/duplicates/YAML features, unknown names/types,
non-yielding/unreachable loops, planned budgets versus aborts, town resumption,
profile replacement/rollback and class-ladder equivalence. Preserve inactive-plan
key streams. Record the configuration/block hash, effective parameter epochs,
profile/plan/class selection, current step/repeat state, outcomes and reasons.
The examples are not runnable today and claim no improvement in play.
