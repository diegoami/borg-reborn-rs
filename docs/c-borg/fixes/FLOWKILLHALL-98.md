# source98: judge each monster on its own in borg_flow_kill() (patch 0102)

Consumer adoption of [source 98](https://github.com/diegoami/neo-angband-tracker/issues/98)
(CBUG-0082), fix `69862d0f5` (source PR 473, evidence PR 472), kit
`docs/audit/c-reports/flowkillhall-98` at fixture commit `aed37ebbb`.

## Defect

`borg_flow_kill()` walks the visible monsters, but two pieces of per-monster state were not
reset: `skip_monster` kept the value left by the previous candidate, so **one skipped monster
suppressed every later one**, and `hall_walls` was never reset and its "not in a hallway" test ran
inside the accumulation loops against a partial count. The borg could stop attacking because of an
earlier, unrelated monster.

## Fix

Reset `skip_monster` per monster, reset and total `hall_walls`, and test the total after the
accumulation loops. Source fix `69862d0f5` -> applied `2c96f8d3a` (`cherry-pick -x`), patch
`patches/0102-flowkill-per-monster.patch`, scope `src/borg/borg-flow-kill.c` only.

## Verification

Before commit `9ca986616`, tip `2c96f8d3a`, kit unchanged (`tools/verify-flowkillhall98.py`).
Native gates green at both commits (965/965 unit, 5/5 frontend); byte-wise provenance; the
`src`/`lib`/build scope is the one file.

| Mode | Before | After |
| --- | --- | --- |
| `hall` | **fails** 0/1: the flow still chooses a grid (`flowed=1`) where it should skip | passes 1/1: skip (`flowed=0`) |
| `leak` | **fails** 0/1: an earlier skipped monster leaks into this one (`flowed=0`) | passes 1/1: flow (`flowed=1`) |
| `hallctl` | passes, control | passes, **byte-identical** |
| `reverse` | passes, control | passes, **byte-identical** |

The committed `before-*.log` / `after-*.log` are the product runs; the source kit's own rows are in
`receipt.json`.

## Mode attribution

This is an unconditional change in shared flow code (no mode gate): the fixture proves it at the
function level. The per-mode question - which of normal, munchkin and Lunal play it actually
changes - is a cohort question, deferred to the batch measurement with the zoo's explicit arms
(borg-zoo issue 54); it is recorded as **not measured per mode**, not as neutral.

Matrix group `flowkillhall-98` (4 cases) uses the `after-*.log` rows as its baseline.

## Ledger and credit

`patches/series.json` `FLOWKILLHALL-98`, patch-active; consumer issue #431. Source defect, fix and
native fixture by Diego Amicabile (native proposal/evidence); carried unchanged.
