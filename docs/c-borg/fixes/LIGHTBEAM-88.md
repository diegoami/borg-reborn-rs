# Lightbeam-88 carry: verified product scenario

Consumer [316](https://github.com/diegoami/better-borg-c/issues/316),
canonical source
[88](https://github.com/diegoami/neo-angband-tracker/issues/88)
(CBUG-0072, lightbeam-88; source fix `828318ab5`, evidence fixture
`05e527351`). Applied as a `cherry-pick -x` of source
`828318ab58cbedc3f90cfe1a343ebd314257933b`, patch
`patches/0084-light-beam-panel-edge-row.patch` (applied commit
`ffa3cd8bd1`).

## Defect

`borg_light_beam()` refuses north/south beams "if on the edge of
shifting the panel", but compared the Borg's absolute dungeon row
(`borg.c.y`) with the panel height instead of its row on the panel. On a
22-row panel (an 80x24 terminal) a north or south beam was refused
everywhere below dungeon row 20 — most of a 66-row level — whatever part
of the map was on screen. The correction subtracts `Term->offset_y`
first, as the shift-panel code in `borg-fight-defend.c` does.

## Pins and provenance

Before: merged main `63fbc1fb7` (snapshotted build `build/native-before`,
gate result `artifacts/native-before/result.json`, 965/965 + 5/5 at that
commit). `borg-light.c` is otherwise pristine on the product. The
verifier proves that the source diff applied to the before module equals
the product module and that the applied commit is the only `src`/`lib`
difference from the before commit. Fixture: kit `05e527351` at head
`90e5fc16b`, asserted byte-identical to the run pin.

## Verified results (`artifacts/lb88/scenarios`, 2026-11-08)

- `scrolled` / `scrolled_south` — fail before, pass after: before 0/1
  (`beam_cast=0`; the rule compared the absolute dungeon row and refused
  the beam below dungeon row 20 regardless of the panel scroll), after
  1/1 (`beam_cast=1`).
- `top` control: 1/1 in both phases, log byte-identical before/after
  (`b1339ef4…`).
- `result.json` records `verified: true` with all commands, pins and
  hashes. The verifier passed on its first complete run.

## Scope and limits

Bounded cast-decision scenarios with the map panel scrolled away from
the dungeon top. They prove the panel-edge rule now tests the row on the
panel; no monster-kill or survival claim.

## Exact next work

1. PR from `task/carry-lightbeam`, agent evidence review plus required
   hosted checks, then normal merge.
2. Frozen v1 cohort at the merged tip against the mod-neck tip cohort.
3. Receipts on issue 316, source 88 and checkpoint 1.
