# Errands-plan POC examples

`errands_plan.txt` contains explicit schedules for all nine classes and a
default. They cover levels 1–49 and hand back on reaching 50; they do not
promise reaching it. Early class gates follow the old ladder groups; later
intervals are shared, unvalidated hypotheses pending evaluation.
It is plain text: class sections, level ranges, and exactly one supported
errand, `loop-levels`. No ladder/trips aliases or general scripting language.

See [the POC guide](../../docs/GAME-PLAN-LANGUAGE.md) for actual launch,
validation and boundaries. Select a file using `BORG_ERRANDS_PLAN` or
`borg_plan = errands_plan.txt` in the run's borg.txt (relative to its user
directory). It is read once on the borg-rs backend; no selection is inert.

Use `tools/try-errands-plan.py` for bounded diagnostics. Plan runs are not yet
compatible with normal measured-arm/death-rerun identity, and the standard
harness rejects them pending that integration. The POC does not modify the
currently running recorded game. Existing original-core BORG_PROFILE and zoo
profile JSON are separate interfaces, not extra plan formats.
