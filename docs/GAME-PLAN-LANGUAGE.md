# Errands plan POC: class-specific level phases

PR #308 is now a **working, opt-in POC**, not a full-language design. It tests
one separate plain-text `errands_plan.txt`, class selection and phase changes
in the real Borg. No YAML, embedded EOF block, profiles DSL or unique tactics.

## The file

The full candidate file is
[`harness/profiles/borg-rs/errands_plan.txt`](../harness/profiles/borg-rs/errands_plan.txt).
It contains explicit schedules for all nine classes and one default. Every character level 1–49 has one phase; reaching level 50
intentionally hands back, pending the capability-based follow-up. This does
not guarantee reaching level 50. The small block below illustrates the format,
not the full candidate file:

```text
[default]
  level <= 5
    loop-levels 0 2
  5 < level <= 10
    loop-levels 0 4
  10 < level <= 15
    loop-levels 2 6

[mage]
  level <= 5
    loop-levels 0 1
  5 < level <= 10
    loop-levels 0 2
  10 < level <= 15
    loop-levels 1 3
```

A matching class section replaces the default **completely**. Its gaps do not
fall through to the default. Classes without a section use `[default]`; with
neither, startup fails visibly. Supported class names are `warrior`, `mage`,
`druid`, `priest`, `necromancer`, `paladin`, `rogue`, `ranger`, `blackguard`.
Early gates in the full file follow the two groups in the existing
owner-tuned ladders. Later intervals are shared hypotheses, not nine invented
class calibrations. They do not override native preparedness. Later loops
omit compulsory town endpoints and retain core-requested town recovery;
town frequency is an evaluation question, not an established optimum.
These example depths are not calibrated recommendations or improvement claims.

The POC has exactly **one errand: `loop-levels A B`**, reusing the existing
implementation, spelling and readiness policy. Zero is town. `ladder`, `trips`,
`loop-level` aliases and other errands are rejected in this file. This is the
minimum needed to test the model; extra commands must earn their place later.
A phase has one command, not an arbitrary program.

Headers are at column zero, level conditions have two spaces and the command
has four. Blank lines and full-line comments are allowed; tabs, extra commands,
empty/duplicate sections and unknown classes are errors. Conditions accept
`level < N`, `<=`, `>`, `>=`, `==`, or `A < level <= B` / `A <= level <= B`.
Integers must be positive and ranges nonempty/non-overlapping within a class.
Bounds are checked for overflow. Shared boundaries must not overlap: use
`5 < level <= 10` after `level <= 5`. Gaps hand control back with a diagnostic; the shipped file deliberately
has a gap at level 50. Input is limited
to 64 KiB and 256 phases. No imports, nesting, expressions or mutable settings.

## Where it is read

From this POC checkout, a private watched game can also be started with
`BORG_CORE=borg-rs BORG_ERRANDS_PLAN="$PWD/harness/profiles/borg-rs/errands_plan.txt" ./watch-borg.sh --name PlanPOC`.
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
policy decisions. The chosen class is the actual game character's class.
A requested plan with the original reborn backend is rejected rather than
ignored. A nonempty automatic `BORG_ERRAND`/`borg_reborn_errand` conflicts with
the plan. The POC also rejects a nonzero `BORG_ERRAND_BUDGET`; use the bounded
native diagnostic driver below. No hot reload or parameter mutation is provided.
Existing parameters remain in borg.txt; see [CALIBRATION.md](CALIBRATION.md).

## What runs

The controller selects a phase once and retains its loop state. At safe
non-store decision boundaries it checks current character level, not maximum
level reached. Changing phase replaces the active loop once; an unchanged
phase does not restart every think. Experience drain can select an earlier
phase. Pending Recall is allowed to finish before selecting a new phase;
store transactions and partially queued commands are not interrupted.

The existing loop still owns core-required town recovery and restarting its
first endpoint. It does not force an unprepared level. Phase selection clears
old path intent, not the core's safety flags. No match stops with a reason.
`/stop`, a manually selected errand, or an errand's terminating outcome cancels
the plan controller. A different character/older loaded game resets selection.
An ordinary key pause retains existing Borg state; restarting the Borg resumes
it. No in-process plan reload is implemented.

Plan bytes are preserved as `BORG_RUN_DIR/errands_plan.txt` when diagnostics are
enabled. Messages record `# Plan:` with source path, class, level, source line
and installed loop. Existing traces expose that loop's target and decisions.
The original file can be edited without changing the already parsed plan.
A native save alone still does not capture Borg/plan memory; use recordings
for exact lead-ins, not an unqualified fresh-state load claim.

## Run the POC

Build this PR's version with `harness/build.sh` under the shared lock. Then:

```sh
flock /tmp/angband-validation.lock python3 tools/try-errands-plan.py \
  --plan harness/profiles/borg-rs/errands_plan.txt \
  --class 1 --seed 1001 --turns 3000 --out /tmp/my-mage-plan-poc
```

Class 1 is Mage; class 0 is Warrior. `--via-borg-txt` tests the relative-file
setting instead of the environment override. Use a new output directory each
time. The driver preserves exact plan bytes, executable hash, arguments,
settings, console/messages/keys/state, summary and process result in `poc.json`.
No network, engine patch or recorder modification is needed.

This driver is explicitly diagnostic, **not a measured zoo/harness arm**.
The standard `harness/run.sh` refuses plan POC inputs: cemetery encounter-family
and exact-source death reruns do not yet include this new input/controller
state. That integration is required before enabling plans in measured arms;
do not label a plan run as an ordinary model-free baseline or drop plan bytes
from provenance. Diagnostic failures/deaths remain in the preserved output;
no population experiment or survival improvement is claimed.

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
