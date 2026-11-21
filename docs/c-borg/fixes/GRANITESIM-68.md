# source68: test for granite, not granite-or-above, in the ranged move check (patch 0109)

Consumer adoption of [source 68](https://github.com/diegoami/neo-angband-tracker/issues/68)
(CBUG-0052), fix `f1b39e205` (source PR 485, evidence PR 484), kit
`docs/audit/c-reports/granitesim-68` at fixture commit `9954be16d`.

## Defect

The ranged move simulation inside `borg_danger_one_kill()` decides whether a candidate grid can
be occupied:

```c
/* Cannot occupy a wall/seam grid (unless pass_wall or kill_wall) */
if (borg_grids[y_temp][x_temp].feat >= FEAT_GRANITE
    || (borg_grids[y_temp][x_temp].feat == FEAT_MAGMA
        || borg_grids[y_temp][x_temp].feat == FEAT_QUARTZ
        ...
```

Terrain feature values are not ordered; the melee branch of the same function tests
`feat == FEAT_GRANITE`. The `>=` form treats every feature above granite in the enum (rubble and
others) as an impassable wall, so a monster standing on such a grid is invisible to the ranged
simulation: its danger reads zero and the borg does not flee it.

## Fix

Test `feat == FEAT_GRANITE`; the explicit magma/quartz and other seam variants stay listed.
Source `f1b39e205` -> applied `77ec2c55e` (`cherry-pick -x`), patch
`patches/0109-granitesim-ranged-wall.patch`, scope `src/borg/borg-danger.c` only.

## Verification

Before commit `b663b28b0`, tip `77ec2c55e`, kit unchanged (`tools/verify-granitesim68.py`). Native
gates green at both (965/965 unit, 5/5 frontend); byte-wise provenance over the source diff; the
`src`/`lib`/build scope is the one file.

| Mode | Before | After |
| --- | --- | --- |
| `rubble` (monster on rubble) | **fails** 0/1: danger 0, no action | passes 1/1: danger 208, fleeing condition, Recall |
| `floor` (monster on open floor) | passes, control: danger 208 | passes, **byte-identical** |
| `granite` (monster behind granite) | passes, control: danger 0 | passes, **byte-identical** |

The committed `before-*.log` / `after-*.log` and `result.json` are the product runs; the source
kit's own rows and file hashes are in `receipt.json`, carried unchanged.

## Mode attribution

Unconditional change in the shared danger model (no mode gate): the ranged simulation runs for
every shooting decision in every mode, and the fixture shows the affected branch executing on the
rubble grid while the granite grid keeps its old value. The per-configuration cohort effect is
recorded as **not measured**, not neutral, and belongs to the batch with the zoo's explicit arms
(borg-zoo issue 54). This is the third and last of the three `borg-danger.c` fixes in the queue;
the batch notes they compound, so attributing an effect requires measuring them one at a time on
the same arm.

Matrix group `granitesim-68` (3 cases) uses the `after-*.log` rows as its baseline. Full matrix
at this tree: 87 groups / 268 cases, `ok: true`, no failed group, 215/228 byte-identical; the
13 byte differences are in the same six groups as before this carry, so no new difference.

## Ledger and credit

`patches/series.json` `GRANITESIM-68`, patch-active; consumer issue #452. Source defect, fix and
native fixture by Diego Amicabile (native proposal/evidence); carried unchanged.
