# source66: clear the ranged danger when Crush would kill the monster (patch 0108)

Consumer adoption of [source 66](https://github.com/diegoami/neo-angband-tracker/issues/66)
(CBUG-0050), fix `9757b1617` (source PR 481, evidence PR 479), kit
`docs/audit/c-reports/crushv2-66` at fixture commit `5ad69b410`.

## Defect

`borg_danger_one_kill()` computes several danger values per candidate grid: `v1` for the melee
value and `v2` for the ranged one. When a Crush-style kill is judged sufficient to finish the
adjacent monster, the code clears the grid's danger so the borg can take the kill - but the
second block assigns the wrong variable:

```c
if (ag->info & BORG_VIEW && borg_cave_floor_grid(ag))
    v1 = 0;
```

`v1` is cleared again instead of `v2`. The ranged danger therefore keeps the adjacent monster's
value, and a caster or archer whose best kill is Crush declines and does nothing.

## Fix

Assign `v2 = 0`. Source `9757b1617` -> applied `acc994185` (`cherry-pick -x`), patch
`patches/0108-crushv2-ranged-danger.patch`, scope `src/borg/borg-danger.c` only.

## Verification

Before commit `0b5889caf`, tip `acc994185`, kit unchanged (`tools/verify-crushv266.py`). Native
gates green at both (965/965 unit, 5/5 frontend); byte-wise provenance over the source diff; the
`src`/`lib`/build scope is the one file.

| Mode | Before | After |
| --- | --- | --- |
| `caster` | **fails** 0/1: no action, keys 0, monster alive | passes 1/1: Crush cast, keys 3, monster dead |
| `archer` | **fails** 0/1: no action, keys 0, monster alive | passes 1/1: Crush cast, keys 3, monster dead |
| `melee` | passes, control | passes, **byte-identical** |
| `tough` | passes, control (Crush is not enough) | passes, **byte-identical** |

The committed `before-*.log` / `after-*.log` and `result.json` are the product runs; the source
kit's own rows and file hashes are in `receipt.json`, carried unchanged.

## Mode attribution

Unconditional change in the shared danger model (no mode gate): the function runs for every
ranged attack decision in every mode, and the fixture shows the affected branch executing in the
caster and archer modes. The per-configuration cohort effect is recorded as **not measured**,
not neutral, and belongs to the batch with the zoo's explicit arms (borg-zoo issue 54). This is
the second of the three `borg-danger.c` fixes; the batch notes they compound, so attributing an
effect requires measuring them one at a time on the same arm.

Matrix group `crushv2-66` (4 cases) uses the `after-*.log` rows as its baseline. Full matrix
at this tree: 86 groups / 265 cases, `ok: true`, no failed group, 212/225 byte-identical; the
13 byte differences are in the same six groups as before this carry, so no new difference.

## Ledger and credit

`patches/series.json` `CRUSHV2-66`, patch-active; consumer issue #450. Source defect, fix and
native fixture by Diego Amicabile (native proposal/evidence); carried unchanged.
