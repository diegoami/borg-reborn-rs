# A level-based Borg plan inside borg.txt

Status: revised design proposal for PR #308, 2026-11-15, based on the owner's
concrete example. Replaces both the general language and YAML proposals.
No parser, runner or live configuration change is implemented here.

## The format

```text
borg_plan = <<EOF
  level <= 5
    loop-level 0 2
  5 < level <= 10
    loop-level 0 4
  10 < level <= 15
    loop-level 2 6
EOF
```

This is plain indented plan text, not YAML/JSON/TOML. The lower bounds are
exclusive because the owner's original shared boundaries would match two
phases at levels 5 and 10. Reject overlapping ranges rather than introduce
hidden ordering rules. The depths express the example, not a measured playing
recommendation.

Through character level 5, explore between town and DL2. At levels 6–10,
explore between town and DL4. At levels 11–15, explore between DL2 and DL6.
Each loop explores intermediate levels in both directions, subject to the
core's readiness and safety. Zero is town. `loop-level` is the proposed plan
spelling for the existing `/loop-levels` capability, not a second movement
policy or an immediate rename of the slash command. See [ERRANDS.md](ERRANDS.md).

Above level 15 this example has no matching phase and hands back with a reason.
Add `level > 15` with its errands to continue. No `until`, `while`, `repeat`,
variables, imports, arbitrary expressions or nested handlers are needed.

## Where it lives

A Borg profile is the selected **borg.txt configuration**, with ordinary flat
settings and one `borg_plan` block. There is no separate plan-file setting.

- Watched game: `.angband-reborn/user/borg.txt`.
- Guided game: `SESSION/user/borg.txt`, using its recorded manifest.
- Headless run: `OUT/user/borg.txt`, prepared from `harness/borg.txt`.
- Reviewed examples: `harness/profiles/borg-rs/NAME.borg.txt`; select/copy a
  desired configuration when preparing a run. No automatic directory search.

Warrior and mage profiles can contain different schedules without either being
hard-coded into the engine. Existing original-core `reborn-default.txt` /
`BORG_PROFILE` and zoo JSON interfaces keep their separate meanings.
Parameter names and bounds remain in [CALIBRATION.md](CALIBRATION.md).

Current binaries do not support `borg_plan`. Flat parameters still load once
at initialization; editing borg.txt does not currently hot-reload them.

## Parsing and validation

`borg_plan = <<DELIMITER` starts a literal block; an exact `DELIMITER` line in
column zero ends it. `EOF` is the delimiter in the example. Permit a nonempty
ASCII identifier without whitespace. Normalize CRLF for marker comparison;
preserve indentation, with no shell/variable interpolation. A delimiter-like
word on an indented line does not close the block.

Allow one block per file. Every flat-setting reader, including early core/
errand selection, must skip the entire block and its markers. Embedded text
must never become global settings or formula lines. A missing terminator,
duplicate block, invalid requested plan or unsupported syntax fails loudly at
startup. No block preserves existing behaviour. A nonempty automatic
`borg_reborn_errand` conflicts with an automatic plan and is rejected; explicit
interactive replacement is separate.

Ignore blank lines and full-line `#` comments. Phase headers share an
indentation; their commands are indented one level further. Use spaces, reject
tabs/deeper nesting. V1 accepts only integer character-level comparisons:
single lower/upper bounds or chained ranges. Operators are `<`, `<=`, `>`,
`>=`, `==`; no arithmetic, functions, boolean expressions or arbitrary fields.
Compile to integer intervals and reject overlaps. Gaps mean handover, including
when loading a character outside all listed ranges. `level` is current level;
experience drain can select an earlier phase, not silently the highest phase.

Command lines use existing primitive errands and checked arguments: `town`,
`shopping`, `walk-back`, `explore-back`, `dive N`, `explore-level`,
`dive-exploring N`, and `loop-level A B`. Several errands may run in sequence;
an endless errand must be last. Reject empty phases and unreachable steps.

## Execution

Select the matching phase at start. While it still matches, retain its current
errand/cursor; do not restart the first command every think. Finite errands
advance once. Completing all commands marks the phase finished and hands back,
rather than repeating shopping automatically.

Recheck the level guard at every safe decision boundary, **even while an
endless loop errand runs**. A different phase cancels the previous strategic
errand and starts the new phase's first command. The user need not add an
artificial turn budget just to detect a level-up. Complete a pending prompt,
multi-key command or store transaction before switching. Pending Recall and
emergency recovery finish safely; never cancel Recall or queue commands into
its prompts to satisfy a phase transition.

Core emergencies and town intent have priority. Pause for town recovery, then
reevaluate the phase before resuming. `loop-levels` already owns its town detour
and first-endpoint restart: observe it instead of adding a duplicate trip.
Readiness still prevents forced descent. Manual stop cancels the plan.
Abort/error/scared and no matching phase hand back with a visible reason.
One strategic errand owns the game; commands do not execute concurrently.

## Parameters, ladders and later tactics

The first delivery should implement the exact phase/errand format above.
Later, permit existing `borg_param_<name> = INTEGER` assignments in a phase,
using the existing registry rather than another expression language. These
would be phase overrides over the flat configuration, removed on leaving the
phase. Explicit environment run locks remain authoritative with visible
conflict diagnostics.

Live updates need a separate mutability/cache audit and atomic apply/rollback;
startup-only keys remain fixed. No backend, birth-option or engine-constant
changes. Do not promise arbitrary mid-fight settings just because a block
parser exists, or introduce nested YAML profiles to implement them.

Move class ladders and trip schedules into the relevant Borg profiles.
`/ladder` can remain a compatibility alias for the selected profile's plan.
Relocate legacy data behind its compatibility adapter first; retire tables/
executors only after the reduced format represents the semantics and class
checks preserve readiness, skip rules, level gates, last-trip repetition and
town recovery. Keep an explicitly deprecated adapter for unsupported legacy
cases instead of calling an approximate schedule equivalent. No duplicate
sources of ladder policy or growing a second `/trips` language.

Reviewed named unique tactics and preparation kits come later through accepted
combat/item/spell APIs, with a separate concrete design and tests. The
[adviser proposal #309](https://github.com/diegoami/borg-reborn-rs/pull/309)
must not smuggle arbitrary scripts into this small configuration format.

## Records, delivery and checks

Freeze configuration bytes/hash at start. Explicit reload validates the whole
file first, then restarts the matching phase safely and records the reason;
file edits do not silently alter a live plan. Record phase/line, errand cursor,
level changes, interruption/outcome reasons and later parameter epochs. An
Angband save alone cannot restore Borg/plan memory: require qualified joint
checkpoint/replay or label a fresh-state restart honestly. Preserve recordings.

Deliver block validation, then phase selection/active-errand cancellation,
then compatibility-preserving ladder migration. Parameter updates and tactics
are separate later slices. Reuse accepted R control APIs and P's registry;
inventory/tactics depend on #293/R2/R3/R4 capabilities. Track B remains undefined
in main and needs clarification before scheduling. No milestone or contract
amendment is activated by this design draft.

Checks: flat-file compatibility/block isolation, CRLF/truncation/delimiters,
overlaps/gaps, levels 5/6/10/11/15/16, experience drain, endless-child phase
change, completed phases not restarting, town recovery during a phase change,
prompts/Recall, manual stop, ladder equivalence and inactive-plan key parity.
No interpreter, live setting mutation or improved play is claimed here.
