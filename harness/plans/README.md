# Errands-plan POC examples

`errands_plan.txt` contains explicit schedules for all nine classes and a
default. They cover levels 1–49 and hand back on reaching 50; they do not
promise reaching it. Early class gates follow the old ladder groups; later
intervals are shared, unvalidated hypotheses pending evaluation.
It is plain text: class sections, level ranges, and exactly one supported
command per phase: `max-depth N` (a depth cap; the core plays as itself) or
`loop-levels A B` (a forced route). Both shipped plans use only `max-depth`
since 2026-11-16 (owner): `loop-levels A B` became `max-depth B`. The
loop-levels form of the ambitious plan is kept as a test fixture
(`engine/tests/fixtures/loop-levels-ambitious-plan.txt`). No ladder/trips
aliases or general scripting language.

See [the POC guide](../../docs/GAME-PLAN-LANGUAGE.md) for actual launch,
validation and boundaries. Select a file using `BORG_ERRANDS_PLAN` or
`borg_plan = errands_plan.txt` in the run's borg.txt (relative to its user
directory). It is read once on the borg-rs backend; no selection is inert.

`errands_plan_ambitious.txt` (#352) is a second, deeper hypothesis from the
Strategy Manual's depth curve; see the guide.

Measured runs: `BORG_CORE=borg-rs BORG_ERRANDS_PLAN=harness/plans/NAME
harness/run.sh ...` puts the plan's bytes into the run's identity (#352).
`tools/try-errands-plan.py` remains for bounded diagnostics. The zoo's
adapter support is requested in borg-zoo#123. Existing original-core
BORG_PROFILE and zoo profile JSON are separate interfaces, not plan formats.
