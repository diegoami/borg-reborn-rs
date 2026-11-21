# source95: keep the wall under a newly seen wall-walking monster (patch 0118)

Consumer adoption of [source 95](https://github.com/diegoami/neo-angband-tracker/issues/95)
(CBUG-0079), fix `1a220cb2d` (source PR 476, evidence PR 475), kit
`docs/audit/c-reports/newkill-95` at fixture commit `54723234f`.

## Defect

`borg_new_kill()` closed by forcing the monster's grid to floor, reading the race from the
tracking slot *before* the new race was stored in it:

```c
/* Access the monster */
kill  = &borg_kills[n];
r_ptr = &r_info[kill->r_idx];
ag    = &borg_grids[y][x];

/* Save the race */
kill->r_idx = r_idx;
...
/* Force the monster to be sitting on a floor
 * grid unless that monster can pass through walls
 */
if (!(rf_has(r_ptr->flags, RF_PASS_WALL))) {
    ag->feat = FEAT_FLOOR;
}
```

`kill->r_idx` is still the slot's previous race there - race 0 for a wiped slot - so the step
always marked the grid as floor. A pass-wall monster first seen inside seen rock therefore turned
that rock into floor in the Borg's map; the "never shoot walls" check then let the Borg fire, the
game stopped the missile in front of the wall, and the turn and the shot were lost for 0 damage.

## Fix

`borg_update_kill_old(n)`, called a few lines earlier with the correct race, already performs that
step. Source `1a220cb2d` -> applied `2b23139bc` (`cherry-pick -x`), patch
`patches/0118-newkill-keep-wall.patch`, scope `src/borg/borg-flow-kill.c` only.

## Verification

Before commit `c1177a2df`, tip `54723234f`, kit unchanged (`tools/verify-newkill95.py`). Native
gates green at both (965/965 unit, 5/5 frontend); byte-wise provenance over the source diff; the
`src`/`lib`/build scope is the one file.

| Mode | Before | After |
| --- | --- | --- |
| `wall` (Human Warrior, seed 1000, level 1, depth 20, sling + iron shots, earth spirit in a granite pillar) | **fails** 0/1: the map says floor, the Borg fires, 0 damage, turn and shot lost | passes 1/1: the map keeps the granite, no shot is queued |
| `floor` (the same earth spirit on open floor) | passes, control: fires and hits for 4 | passes, **byte-identical** |

The committed `before-*.log` / `after-*.log` and `result.json` are the product runs; the source
kit's own rows and file hashes are in `receipt.json`, carried unchanged. All four product logs are
byte-identical to the source lane's captured logs for the same fixture, so the kit reproduces
unchanged on this tree.

## Mode attribution

The sighting path is shared code with no mode gate and runs whenever the Borg sees a monster, in
every configuration. The change only bites when a pass-wall monster is first seen inside seen rock;
the fixture shows the wall mode flipping while the open-floor control keeps its log. Per-
configuration cohort movement is recorded as **not measured**, not neutral, and belongs to the batch
with the zoo's explicit arms (borg-zoo issue 54).

Matrix group `newkill-95` (2 cases) uses the `after-*.log` rows as its baseline. Full matrix
at this tree: 96 groups / 292 cases, `ok: true`, no failed group, 239/252 byte-identical; the
13 byte differences are in the same six groups as before this carry (`dispel`, `dispel259`,
`fleeheal-408`, `knockout`, `preparation`, `vampire`), so no new difference.

## Ledger and credit

`patches/series.json` `NEWKILL-95`, patch-active; consumer issue #475. Source defect, fix and
native fixture by Diego Amicabile (native proposal/evidence); carried unchanged.
