# Torodvalue-549 carry: verified product scenario

Consumer [361](https://github.com/diegoami/better-borg-c/issues/361),
canonical source
[549](https://github.com/diegoami/neo-angband-tracker/issues/549)
(torodvalue-549; source fix `d4071b06e`, evidence fixture `f3323c2f0`).
Applied as a `cherry-pick -x` of source
`d4071b06e2d298a17e1a55559daceccf9663f9f3`, patch
`patches/0091-tele-other-rod-defence.patch` (applied commit
`187bb3b61e`).

## Defect

The defence that teleports a dangerous monster away accepted the
Teleport Other spell, an item activation or a charged Wand of Teleport
Other, but not a Rod of Teleport Other. A Borg without the spell that
carried only the rod never considered teleporting the monster away: a
wounded Warrior next to a hill giant kept meleeing it. The correction
accepts a charged rod the Borg can zap (`borg_equips_rod`, the same
device-skill check used for other rods) and zaps it when the defence is
chosen, after the spell, activation and wand; the target key that
follows is the same.

## Pins and provenance

Before: merged main `820bc0cbb` (snapshotted build `build/native-before`,
gate result `artifacts/native-before/result.json`, 965/965 + 5/5 at that
commit). `borg-fight-defend.c` is otherwise pristine on the product. The
verifier proves that the source diff applied to the before module equals
the product module and that the applied commit is the only `src`/`lib`
difference from the before commit. Fixture: kit `f3323c2f0` at head
`8b30cf629`, asserted byte-identical to the run pin.

## Verified results (`artifacts/tr549/scenarios`, 2026-11-09)

- `rod` — fail before, pass after: before no Teleport Other was used
  and the monster stayed (`used_teleport_other=0, monster_moved=0`;
  the rod was invisible to the defence), after the rod is zapped and
  the monster moves (`=1, =1`).
- `wand` control: 1/1 (wand zapped, monster moved) in both phases, log
  byte-identical before/after (`35dd9c4e…`).
- `weak` control: 1/1 (defence correctly not fired) in both phases,
  log byte-identical before/after (`450a4698…`).
- `result.json` records `verified: true` with all commands, pins and
  hashes. The verifier passed on its first complete run.

## Scope and limits

Bounded defence scenarios (rod only, wand control, and a weak-Borg
control where the defence must not fire). They prove the rod acceptance
and zap; no survival claim.

## Exact next work

1. PR from `task/carry-torodvalue`, agent evidence review plus required
   hosted checks, then normal merge.
2. Frozen v1 cohort at the merged tip against the starvealch tip cohort.
3. Receipts on issue 361, source 549 and checkpoint 1.
