# Errands plan POC: class-specific level phases

PR #308 is now a **working, opt-in POC**, not a full-language design. It tests
one separate plain-text `errands_plan.txt`, class selection and phase changes
in the real Borg. No YAML, embedded EOF block, profiles DSL or unique tactics.

## The file

The full candidate file is
[`harness/plans/errands_plan.txt`](../harness/plans/errands_plan.txt).
It contains explicit schedules for all nine classes and one default. Every character level 1–49 has one phase; reaching level 50
intentionally hands back, pending the capability-based follow-up. This does
not guarantee reaching level 50. The small block below illustrates the format,
not the full candidate file:

```text
[default]
  level <= 5
    max-depth 2
  5 < level <= 10
    max-depth 4
  10 < level <= 15
    max-depth 6

[mage]
  level <= 5
    max-depth 1
  5 < level <= 10
    max-depth 2
  10 < level <= 15
    max-depth 3
```

A matching class section replaces the default **completely**. Its gaps do not
fall through to the default. Classes without a section use `[default]`; with
neither, startup fails visibly. Supported class names are `warrior`, `mage`,
`druid`, `priest`, `necromancer`, `paladin`, `rogue`, `ranger`, `blackguard`.
Early gates in the full file follow the two groups in the existing
owner-tuned ladders. Later intervals are shared hypotheses, not nine invented
class calibrations. They do not override native preparedness. There are
no planned town visits; town frequency is the core's own, an evaluation
question, not an established optimum.
These example depths are not calibrated recommendations or improvement claims.

A phase has exactly one command (owner, 2026-11-16).

**`max-depth N`** is a cap, not a route.
The phase installs no errand: the core plays as itself, with its own dives,
town trips, shopping and recovery, and is never prepared for a level below
DL N. The active phase is a separate limit: preparedness and depth correction
read the minimum of the current configured `borg_no_deeper` and N+1. The plan
never overwrites configuration. Reloading borg.txt or reopening the same
character retains the phase limit while honoring the freshly loaded value.
Hand-back at a gap (such as level 50), `/stop` or a manual errand clears the
phase limit immediately, including while Recall is pending. Phase changes
preserve the core's active flow and do not install or clear errands.
The core's own cap applies as usual: when it leaves a level by the
nearest stair it may step onto the level below and climbs back at once
("Needing to get back on correct depth"); it never stays there. N is 1–126.
Town trips are the core's own, plus the M13 trip policy when it is on.

`loop-levels A B`, the earlier forced route, was removed with the
level-loop errand: a plan that uses it fails with "loop-levels was removed;
use max-depth N" (exit 2). `/max-depth`, `MAX-DEPTH`, `max-depth:N` and
other errands are rejected. A phase has one command, not an arbitrary program.

Headers are at column zero, level conditions have two spaces and the command
has four. Blank lines and full-line comments are allowed; tabs, extra commands,
empty/duplicate sections and unknown classes are errors. Conditions accept
`level < N`, `<=`, `>`, `>=`, `==`, or `A < level <= B` / `A <= level <= B`.
Integers must be positive and ranges nonempty/non-overlapping within a class.
Bounds are checked for overflow. Shared boundaries must not overlap: use
`5 < level <= 10` after `level <= 5`. Gaps hand control back with a diagnostic; the shipped file deliberately
has a gap at level 50. A leading UTF-8 BOM and CRLF files are accepted. Input is limited
to 64 KiB and 256 phases; size is checked before decoding UTF-8. No imports, nesting, expressions or mutable settings.

## Where it is read

From this POC checkout, a private watched game can also be started with
`BORG_CORE=borg-rs BORG_ERRANDS_PLAN="$PWD/harness/plans/errands_plan.txt" ./watch-borg.sh --name PlanPOC`.
This is a new game/process; it does not hot-patch a running recorded binary.

Interactive selection through the vendored core's configuration:

```text
# In the actual run's user/borg.txt:
borg_reborn_core = borg-rs
borg_plan = errands_plan.txt
```

Put that `errands_plan.txt` beside this borg.txt. Relative paths resolve under
`ANGBAND_DIR_USER`, **not** the process's current directory. Absolute paths
are accepted. `BORG_ERRANDS_PLAN=/absolute/path/errands_plan.txt` overrides the
file setting. An empty explicit path is an error; with no selection, existing
behaviour is unchanged. No automatic search or YAML/TOML profile loader.

For normal watched games the files are `.angband-reborn/user/borg.txt` and
`.angband-reborn/user/errands_plan.txt`. A guided game uses its own
`SESSION/user/` directory from the manifest. Do not change the currently
running frozen recording: it has the old executable. A new process/segment
using a POC build is required.

The file is loaded and validated once during core initialization, before
policy decisions; the actual class must resolve to its own or default section
before the preserved-plan acknowledgement is written. This is checked again
on core reinitialization for another character. The chosen class is the actual game character's class.
An environment-requested plan with the original reborn backend is rejected.
A file-selected original core also rejects a file plan. Explicit
`BORG_CORE=reborn` retains lazy environment priority and does not consult
borg.txt, including its plan setting; use the vendored core for file plans. A nonempty automatic `BORG_ERRAND`/`borg_reborn_errand` conflicts with
the plan. The POC also rejects `BORG_ERRAND_BUDGET` except absent, a zero integer or
exact `none` (no whitespace), matching the accepted main parser; use the bounded
native diagnostic driver below. Unreadable/non-UTF-8 borg.txt is diagnosed when plan selection is consulted,
not treated as an absent plan. Missing borg.txt retains absent defaults.
Configuration errors exit 2 at Borg initialization, including in interactive
play: this does not claim unsaved-game recovery. Use the bounded diagnostic
driver before enabling edited settings in a live game. The shared strict-reader startup/re-entry contract is documented in
ERRANDS.md (#317); plan selection uses that same reader. No hot reload or parameter mutation is provided.
Existing parameters remain in borg.txt; see [CALIBRATION.md](CALIBRATION.md).

## What runs

The controller selects a phase once and retains it. At safe
non-store decision boundaries it checks current character level, not maximum
level reached. Changing phase replaces the active cap once; an unchanged
phase does not restart every think. Experience drain can select an earlier
phase. Pending Recall is allowed to finish before selecting a new phase;
store transactions and partially queued commands are not interrupted.

The core owns every move, town trip and recovery; the cap only refuses
levels below it. Phase selection preserves path intent, errands, core goals and safety flags. No match stops with a reason.
`/stop`, a manually selected errand, or an errand's terminating outcome cancels
the plan controller. A different character/older loaded game resets selection.
An ordinary key pause retains existing Borg state; restarting the Borg resumes
it. No in-process plan reload is implemented.

Plan bytes are preserved as `BORG_RUN_DIR/errands_plan.txt` when diagnostics are
enabled. Messages record `# Plan:` with source path, class, level, source line
and installed cap.
The original file can be edited without changing the already parsed plan.
A native save alone still does not capture Borg/plan memory; use recordings
for exact lead-ins, not an unqualified fresh-state load claim.

## Run the POC

Build this PR's version with `harness/build.sh` under the shared lock. Then:

```sh
flock /tmp/angband-validation.lock python3 tools/try-errands-plan.py \
  --plan harness/plans/errands_plan.txt \
  --class 1 --seed 1001 --turns 3000 --out /tmp/my-mage-plan-poc
```

Class 1 is Mage; class 0 is Warrior. `--via-borg-txt` tests the relative-file
setting instead of the environment override. Use a new output directory each
time. The driver preserves exact plan bytes, executable hash, arguments,
settings, console/messages/keys/state, summary and process result in `poc.json`.
No network, engine patch or recorder modification is needed.

The driver is for bounded diagnostics. **Measured runs** use the standard
harness (#352): `BORG_CORE=borg-rs BORG_ERRANDS_PLAN=FILE harness/run.sh ...`
(and so `harness/suite.sh`). `run.sh` copies the plan into the run's
`user/errands_plan.txt` and selects it in the run's `user/borg.txt` with a line
naming the plan's SHA-256, so `borg_txt_sha256`, and with it the cemetery's
encounter family, changes with the plan's bytes (not its file name).
`config.txt` records `borg_errands_plan` (the path relative to the checkout)
and `borg_errands_plan_sha256`. After the run, `run.sh` fails if the game did
not preserve exactly that plan as `errands_plan.txt`. A plan selected inside
the harness's own borg.txt, a missing file or another core is refused.

Still open: the cemetery-owned `rerun-death.py` restores the recorded
environment settings but not yet `BORG_ERRANDS_PLAN`. Until it does, a plan
death's rerun reports *does not reproduce* rather than replaying it; the
dossier's `config.txt` names the plan and its hash. No population experiment
or survival improvement is claimed.

### Two shipped plans

- `harness/plans/errands_plan.txt`: early gates from the class ladder groups,
  shared later bands, many phases starting in town.
- `harness/plans/errands_plan_ambitious.txt` (#352): the Strategy Manual's
  depth curve (ch. 12.2: about one dungeon level per one or two character
  levels to ~DL30, then the DL30–45 shoulder). Fighters (warrior, paladin,
  rogue, ranger, blackguard) take about one level per character level;
  casters and `[default]` start slower. From character level 3 (6 for
  casters) no phase starts in town. An engine test checks that it is
  never shallower than the first plan at any class and level.

Both are hypotheses. The core's readiness still decides each descent.

## Cleanup and next decision

In the new plan format, ladder/trips are gone. Existing slash commands and
`ladders.txt` remain only for current users outside the opt-in POC; removing
those now would change unrelated defaults before validating the replacement.
After the POC is accepted, migrate class schedules into plan files, update
clients, and remove/deprecate the legacy menu/config/executor paths with
explicit diagnostics and semantic checks. No new aliases or competing schedule
sources should be added. Unsupported legacy semantics are not silently
approximated by a level range.

The POC deliberately leaves out named parameter profiles, runtime setters,
unique tactics, LLM control, reload and checkpoint migration. The purpose is
to let the owner try real class-specific schedules and decide the file location,
phase model and minimal errand set before expanding it. Later work still uses
accepted R/P capabilities and review gates; track B remains unclarified.
The separate adviser proposal (#309) is not implemented by this POC.

Next evaluation: [#320](https://github.com/diegoami/borg-reborn-rs/issues/320)
tracks post-50 capability-driven depth, town/equipment policy and staged longer
runs. Zoo checkpoint/resume storage is already tracked in
[borg-zoo #114](https://github.com/diegoami/borg-zoo/issues/114), including exact
continuation qualification. No resumed cohort or storage migration starts here.
