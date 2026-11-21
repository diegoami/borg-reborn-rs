# source100: Shoot'N'Scoot reads the danger of the adjacent monster (patch 0110)

Consumer adoption of [source 100](https://github.com/diegoami/neo-angband-tracker/issues/100)
(CBUG-0084), fix `dbafd291a` (source PR 490, evidence PR 489), kit
`docs/audit/c-reports/scootidx-100` at fixture commit `368e4db99`.

## Defect

`borg_shoot_scoot_safe()` decides whether the Borg can shoot one turn and step to safety next.
Its danger test asks `borg_danger_one_kill()` about the adjacent monster's grid but passes `i` as
the monster argument - the index of the monster in a different list:

```c
else if ((borg_danger_one_kill(
              kill->pos.y, kill->pos.x, 1, i, true, false)
         > borg.avoidance * 3 / 10)
```

The function therefore estimates the danger of `kill->pos` using another monster's data (and, at
the end of the list, an index that does not describe that position). The Borg can shoot and scoot
into a spot the adjacent monster does not actually make safe, or decline one it does.

## Fix

Pass `ag->kill`, the monster whose grid is being evaluated. Source `dbafd291a` -> applied
`7d917489b` (`cherry-pick -x`), patch `patches/0110-scootidx-adjacent-monster.patch`, scope
`src/borg/borg-flow-kill.c` only.

## Verification

Before commit `5f7597bb7`, tip `7d917489b`, kit unchanged (`tools/verify-scootidx100.py`).
Native gates green at both (965/965 unit, 5/5 frontend); byte-wise provenance over the source
diff; the `src`/`lib`/build scope is the one file.

| Mode | Before | After |
| --- | --- | --- |
| `north` (adjacent monster north) | **fails** 0/1: shoots and reads Phase (`scoot=1`, `cast_phase=1`) | passes 1/1: neither (`scoot=0`, `cast_phase=0`) |
| `south` (control) | passes: no scoot | passes, **byte-identical** |
| `pack` (control: scoot still warranted) | passes: scoot taken | passes, **byte-identical** |

The committed `before-*.log` / `after-*.log` and `result.json` are the product runs; the source
kit's own rows and file hashes are in `receipt.json`, carried unchanged.

## Mode attribution

The scoot decision is ranged positioning used whenever the Borg shoots; there is no mode gate.
The fixture drives `borg_shoot_scoot_safe()` directly and shows the north case flip while the
pack case still scoots, so the changed branch executes where the wrong index mattered. The
per-configuration cohort effect (how often an adjacent monster and a scoot opportunity coincide
in unrestricted, munchkin/Lunal and normal play) is recorded as **not measured**, not neutral,
and belongs to the batch with the zoo's explicit arms (borg-zoo issue 54).

Matrix group `scootidx-100` (3 cases) uses the `after-*.log` rows as its baseline. Full matrix
at this tree: 88 groups / 271 cases, `ok: true`, no failed group, 218/231 byte-identical; the
13 byte differences are in the same six groups as before this carry, so no new difference.

## Ledger and credit

`patches/series.json` `SCOOTIDX-100`, patch-active; consumer issue #454. Source defect, fix and
native fixture by Diego Amicabile (native proposal/evidence); carried unchanged.
