# Simple Borg profiles and errand plans

Status: revised design proposal, 2026-11-15, following owner feedback on PR
#308. This replaces the proposed general language with a small plan format.
Nothing here implements live profile switching or changes the current runtime.
Current commands remain documented in [ERRANDS.md](ERRANDS.md).

## The model

A **profile** is one editable file containing parameter overrides and named
plans. A **plan** is a sequence of errands, optionally repeated until a simple
condition holds. A profile can supply different plans/settings for a class.
The engine supplies mechanics and safety; profiles supply the game policy.

Ladders, starting routines, class progression and trip schedules belong in
profiles. They should not become more special commands hard-coded into the
Borg. Keep useful primitives: town, shopping, walk-back, explore-back, dive,
explore-level, dive-exploring and loop-levels. `/ladder` may remain a familiar
alias for the active profile's named `ladder` plan. Existing `/trips` syntax
is a compatibility input, not a second language to extend indefinitely.

No imports, mutable variables, custom functions, general arithmetic, nested
event handlers or new type system. No arbitrary engine actions or raw keys.
Named unique-fight tactics and LLM advice are later, separate capabilities.

## Where to read and put profiles

Proposed shared locations:

```text
harness/profiles/
  reborn-default.txt            # existing original-core format; unchanged
  borg-rs/
    README.md                  # proposed profile format and storage contract
    normal.toml                # reviewed defaults + ordinary named plans
    cautious.toml              # separately reviewed settings/plans
```

Personal profiles belong in `profiles/` under the actual game's user directory:

```text
.angband-reborn/user/profiles/my-plan.toml                  # normal watched game
SESSION/user/profiles/my-plan.toml                         # guided/private game
```

`SESSION` is the session directory in its manifest, not a global save location.
An initial standalone plan may also be selected by an explicit host-supplied
file path, but the normal editing workflow is a single profile file.

Resolution is explicit and small:

- `profile cautious` selects shared `harness/profiles/borg-rs/cautious.toml`.
- `profile user:my-plan` selects `ANGBAND_DIR_USER/profiles/my-plan.toml`.
- Personal files do not silently shadow shared names. Names are simple file
  basenames; paths/traversal and network lookup are not allowed in a plan.
- Select a named plan from that file, normally `start`. A missing requested
  file/plan, unknown backend/class override or invalid setting is an error.

These paths and commands are a proposal; the current Borg does **not** load
these TOML files yet. Today, read the core parameter catalogue and bounds in
[CALIBRATION.md](CALIBRATION.md). The vendored core reads settings once at
initialization from its run's `ANGBAND_DIR_USER/borg.txt`:

```text
borg_param_caution_flee_danger_hp_multiplier = 1
```

That value is an illustration, not a recommended preset. An explicit
`BORG_PARAM_*` environment value overrides borg.txt. Editing the file while
playing does not currently hot-reload the parameter table. Existing
`BORG_PROFILE` and `harness/profiles/reborn-default.txt` configure the original
reborn core; they are not the vendored core's dynamic-profile loader.
Zoo catalogue profile JSON is another existing interface, documented in
[ZOO-INTERFACE.md](ZOO-INTERFACE.md); do not confuse it with these player files.

## One profile file

Use standard TOML for settings and multiline strings for plans. No separate
manifest, imports or inheritance hierarchy:

```toml
version = 1
backend = "borg-rs"

[parameters]
# Canonical names from CALIBRATION.md; bounds come from the existing registry.
caution_flee_danger_hp_multiplier = 1

[plans]
start = """
town resume
repeat until level >= 5:
    town
    profile cautious
    shopping
    loop-levels 0 2 for 10000 turns
town
profile normal
loop-levels 3 5
"""

# Optional, single-level class override; no recursive inheritance.
[classes.warrior.plans]
start = """
town
shopping
loop-levels 0 2
"""
```

The plan is illustrative, not executable today or a claim of improved play.
Canonical parameter names are translated to the existing borg.txt key form
when preparing startup configuration; profile metadata is not passed through
as unrecognised Borg settings. Defaults plus the profile's parameter table,
then its matching class overrides, form a complete effective configuration.
Absent class entries use the common values/plans. Unknown class names fail
validation rather than silently falling through.

Switching settings with `profile normal` does not jump into normal's `start`
plan. The current plan continues at the next line. Invoking a different named
plan is an explicit player command, proposed as `/plan NAME`; choosing initial
profile/plan is likewise explicit host configuration. Exact launch/UI spelling
is an implementation detail to settle before delivering a loader.

## The entire first plan format

One command per line; `#` comments and blank lines are ignored. Indentation
is four spaces; tabs are rejected. Version 1 has only these constructs:

| Construct | Meaning |
| --- | --- |
| An existing primitive errand and its arguments | Run it to completion using the existing core |
| `ERRAND for N turns` | Run a bounded chunk; N is a positive game-turn budget |
| `profile NAME` | Apply that named profile's settings between errands |
| `repeat until level >= N:` | Check before each iteration; repeat its indented errands until character level reaches N |
| `repeat until max-depth >= N:` | The same, using the character's reached depth |
| `repeat until ready-for N:` | The same, using the Borg's existing readiness judgement |
| `repeat forever:` | Repeat a yielding sequence indefinitely |
| `town resume` or `town stop` | One top-level policy for a core-requested town detour; default resume |

There are no arbitrary boolean expressions or nested repeat blocks in v1.
Conditions are checked **between errands**, not continuously inside them.
An endless errand inside an `until` block must have a `for` budget, otherwise
it would prevent checking the condition again. Reject that plan on loading.
An endless final errand or a final `repeat forever` is allowed; reject any
following unreachable steps. A repeat must contain an errand that yields to
the game; empty/profile-only loops are invalid. The host also bounds control
steps per game decision, so an invalid plan cannot busy-loop without turns.

A bounded chunk reaching its configured budget is an expected yield for this
plan runner, so it advances to the next line/check. `scared`, unexpected abort,
impossible action and errors stop the plan and hand control back with a reason.
This requires an explicit adapter: existing slash-command queues still drop
remaining commands on budget/abort, as documented today. Do not implement
plans by blindly concatenating slash commands and changing their semantics.

## Town interruptions, profiles and restart

The core's emergency/town decisions take priority. With `town resume`, keep
the current plan position, complete the core's town recovery, then retry the
interrupted errand after checking readiness and supplies. With `town stop`,
hand control back. Do not re-enter the same town request every think.
`loop-levels` already owns its town interruption and restarts its first
endpoint: the runner observes it instead of scheduling a duplicate town trip.
Manual stop cancels the plan; it cannot be caught by a repeat.

For the first implementation, profile changes are **town-only**, after the
current command and prompts have finished. A profile line reached in the
dungeon stops with “return to town before switching profile”; there is no
invisible trip to town. Use an explicit `town` line, as in the example.
Switching during a fight is not included in v1.

Selecting a profile replaces the preceding profile's overrides; it does not
accumulate settings from every previously selected file. Apply defaults plus
the selected file/class overrides and explicit run locks. A conflicting
profile value for a locked `BORG_PARAM_*` override is rejected visibly.
Known names, registry bounds, supported mutability and combinations are
validated before changing anything. Commit all settings together, refresh
relevant danger/path/readiness/item caches, and record the effective table
and its hash. Failure leaves the old profile intact. Current parameters are
startup-loaded: even town switching requires that audit/implementation, not
just a TOML parser. Initially reject keys not approved for live switching;
keep startup-only keys fixed. Engine constants, birth options and backend
selection are not switchable profile settings.

Profiles are read and validated when the plan starts; pin the bytes/hashes of
every referenced profile in the run bundle. File edits take effect only on an
explicit reload at an idle town boundary. Reload validates the complete bundle
first and requires an explicit restart of the selected named plan; it does not
try to migrate a half-executed plan. Record old/new hashes and the restart.
Pause/resume without reload retains the same plan position and frozen files.
A native Angband save alone cannot restore the Borg's memory or plan cursor;
use a qualified joint checkpoint/replay, or label a fresh-state restart honestly.

## Move existing policy into profiles without changing it by accident

1. Extract class ladder schedules and their level/readiness gates from
   `borg-core/ladders.txt` into profile-owned data/plans. Keep a compatibility
   adapter to the existing executor while doing this; avoid two authoritative
   copies. Preserve existing `/ladder` behaviour until equivalence is checked.
2. `/ladder` resolves the active profile/class `ladder` plan. Move class-specific
   shopping/progression choices into the same files as their capability APIs
   become available. A different schedule is an explicit different profile.
3. Deprecate global ladder/trips configuration once profile replacements cover
   their semantics. Translate old configuration with diagnostics; do not
   silently change skip/unprepared rules, last-trip repetition or town recovery.
4. Remove the legacy executor only after the simple plan format can represent
   the old behaviour and fixed-start/class tests confirm it. If it cannot,
   retain the deprecated adapter and state the unsupported conversion instead
   of adding a general language to v1 or calling an approximation equivalent.

Later, add names of reviewed unique-fight tactics to profiles: a preparation
kit, engagement requirement and named fight/retreat behaviour. Item counts,
spell choices and action recipes need the accepted inventory/tactical APIs
and their own design/tests; no tactical scripting syntax is proposed here.
The LLM adviser in [PR #309](https://github.com/diegoami/borg-reborn-rs/pull/309)
can eventually suggest a reviewed profile/tactic or bounded action through its
own protocol; it does not require turning this plan format into a programming
language.

## Delivery and checks

This design does not start an implementation milestone or amend SPEC/RULES.
First implement static profile/plan validation and inspection, then sequential
errands/repetition and town resume, then audited town-only switching and ladder
migration. Keep fight tactics and active advice in later separate work.

Reuse the accepted P parameter registry and R control interfaces as they become
available, respecting their review gates; inventory directives depend on #293
and R4, and later fight behaviour on R2/R3. Track B remains undefined in current
main (P/R/Q are documented); resolve the owner's reference before assigning
implementation dependencies. Do not create a track or silently equate B with P.

Acceptance checks: reject invalid files/names, bounds, unsupported live keys,
non-yielding/unreachable loops and backend mismatches; prove profile replacement
and failed-switch rollback; distinguish timed yield from abort; resume a town
detour once; preserve all class ladder semantics during migration; preserve
inactive-plan key streams. Record source spans, selected profile/plan/class,
file hashes, effective parameter changes, current line/repeat state, errand
outcomes and interruption reasons. Show these in a simple status/explain view.
Examples and profile names above are design examples, not runnable features or
recommended calibration results. The owner reviews the reduced format before
implementation; no survival improvement is claimed.
