# Design lessons from death and failure analysis

Design-level observations recorded by bug investigations whose evidence showed
no code defect. Not a work queue; nothing here authorizes gameplay changes.
Future redesigns (including the later borg-reborn-rs work described in
UPSTREAM.md) may take them into account; this lane dispatches no Rust work.

## Lesson 1: retreat guard and flee planner must share one danger notion
(2026-11-09, from [#324](https://github.com/diegoami/better-borg-c/issues/324),
evidence `issue-0324-diagnostic/`, verdicts in [#386](https://github.com/diegoami/better-borg-c/issues/386))

D0012: caution retreats above `HP/3` danger while the scary flee flow (CL<=5)
tolerates danger up to `3xHP`, so it routes back through the square caution
just fled - an oscillation with HP bleed against a stationary 2-HP monster
that a single blow would kill. Redesign notes:
- The retreat guard and the flow planner should evaluate the same danger field
  with coordinated thresholds, so "retreat from here" and "route to safety"
  cannot disagree about the same square.
- Stationary never-moving monsters adjacent to a flee path should be priced as
  avoidable targets, not as walk-past tolls.
- Danger estimates may legally be conservative maxima (the `full_damage`
  round-up doubles danger for any >= 1-point speed advantage), but redesigns
  should know the cliff is that sharp; here it widened a retreat that base
  arithmetic would have triggered anyway.
