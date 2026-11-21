# source94: mark the seen unique dead, not the race numbered like the record (patch 0122)

Consumer adoption of [source 94](https://github.com/diegoami/neo-angband-tracker/issues/94)
(CBUG-0078), fix `ea3e72139` (branch `task/uniquedead-94`), kit
`docs/audit/c-reports/uniquedead-94` from the evidence branch `task/uniquedead-94-evidence`.

## Defect

`borg_update_kill_old(i)` has a fix-up for a monster whose race is a unique the player has already
killed: it marks that unique as dead in `borg_race_death[]`. It indexed the table with `i`, the
Borg's monster-record index, instead of `kill->r_idx`, the race index:

```c
if ((rf_has(r_ptr->flags, RF_UNIQUE)) && r_ptr->max_num == 0)
    borg_race_death[i] = 1;
```

So it marked whichever race happens to have the record's number as dead and left the seen unique
alone; the mark is never cleared during the session. It is reached in normal play when a monster
takes the shape of a killed unique (a Maia of Aule can change into Quaker, Master of Earth). With the
record numbered 47, Grip, Farmer Maggot's Dog, still alive, was recorded dead; the next unique scan
dropped it from the living uniques, which moved the depth the recall-depth decision in
`borg_recall()` is based on.

## Fix

`borg_race_death[kill->r_idx] = 1`: index the table with the race that was seen. Source `ea3e72139` ->
applied `3e7fefb2f` (`cherry-pick -x`), patch `patches/0122-uniquedead-race-index.patch`, scope
`src/borg/borg-flow-kill.c` only (one line).

## Verification

Before commit `e4c221fee` (the kit commit, `src/` identical to `origin/main`), tip `3e7fefb2f`, kit
unchanged (`tools/verify-uniquedead94.py`). Native gates green at both (965/965 unit, 5/5 frontend);
byte-wise provenance over the source diff; the `src`/`lib`/build scope is the one file.

| Mode | Before | After |
| --- | --- | --- |
| `grip` (Maia in Quaker's shape becomes record 47, Grip's race number; Human Warrior, seed 94, level 31, depth 15, maximum and recall depth 60) | **fails** 0/1: Grip marked dead (`grip_dead=1`), 56 living uniques, hunted depth 41, the Borg answers 'n' (`reset_recall=0`, recall depth stays 60) | passes 1/1: Quaker marked, Grip alive, 57 living uniques, hunted depth 17, answers 'y' (`reset_recall=1`, recall depth 15) |
| `slot5` (the same sighting becomes record 5, a non-unique race), control | passes | passes, **byte-identical** |

The committed `before-*.log` / `after-*.log` are the source lane's captures; the product runs of this
tree (`product-result.json`) are byte-identical to all four, so the kit reproduces unchanged. The
source's own rows and hashes are in `receipt.json`, carried unchanged.

## Mode attribution

Per the 2026-11-10 owner rule, each configuration is stated, and "neutral" is claimed only where the
changed branch executes. The changed line has no mode gate and executes in every configuration
(unrestricted, munchkin/Lunal-only, normal-only), but only when the Borg sees a monster whose race is a
unique the player has already killed, typically a shape-changer such as a Maia of Aule. A run in which
no unique has been killed cannot reach it, and it needs the killed unique's shape to be seen again, so
a short fresh-start cohort (such as the zoo class grid) is unlikely to execute it; that is not a
neutrality measurement. The per-configuration effect is **not measured** and belongs to the batch with
the zoo's explicit arms (borg-zoo issue 54).

Matrix group `uniquedead-94` (2 cases) uses the `after-*.log` rows as its baseline.

Full matrix at this tree: 100 groups / 307 cases, `ok: true`, no failed group, 254/267
byte-identical; the 13 byte differences are in the same six groups as before this carry (`dispel`,
`dispel259`, `fleeheal-408`, `knockout`, `preparation`, `vampire`), so no new difference.
