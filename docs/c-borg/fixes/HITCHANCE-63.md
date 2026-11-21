# source63: divide armour by the monster's to-hit in the melee danger estimate (patch 0103)

Consumer adoption of [source 63](https://github.com/diegoami/neo-angband-tracker/issues/63)
(CBUG-0047), fix `5c7950a25` plus the cap `fa5b7e15a` (source PR 500, evidence PR 499), kit
`docs/audit/c-reports/hitchance-63` at fixture commit below.

## Defect

The partial-damage melee estimate computed the chance a monster hits you as

```c
chance = 150 - (((ac * 300) / 4) / 1 + ((r_ptr->level + power) * 3));
```

The armour term is divided by `1` instead of by the monster's to-hit. Armour therefore counts as
if every monster had to-hit 1: for an ordinary monster the expression falls below the 5% floor, the
partial estimate collapses toward 0, and the borg believes a dangerous melee is nearly free. A
separate `borg.near.unique` branch used a different formula for the same quantity. `borg_danger()`
feeds fight, flee and target choice, so this is used in every combat decision.

## Fix

Divide the armour term by the monster's to-hit ((r_ptr->level + power) * 3), drop the unique-only
branch, and cap the partial chance at 100% so it cannot exceed the full blow. Source `5c7950a25`
and `fa5b7e15a` -> applied `99420fc52` and `fa18e1c63` (two `cherry-pick -x` commits, one unit),
patch `patches/0103-hitchance-armour-tohit.patch`, scope `src/borg/borg-danger.c` only.

## Verification

Before commit `281e3dd8d`, tip `fa18e1c63`, kit unchanged (`tools/verify-hitchance63.py`). Native
gates green at both (965/965 unit, 5/5 frontend); byte-wise provenance over the combined source
diff; scope is the one file.

| Mode | Before | After |
| --- | --- | --- |
| `brute` | **fails** 0/1: partial danger 0, full 162, so the sleeping monster is shot (`shoots=1`) and woken | passes 1/1: partial danger 162, no shot (`shoots=0`) |
| `wretch` | passes, control: still worth shooting | passes, **byte-identical** |

## Mode attribution

Unconditional change in the shared danger model, so there is no mode gate to prove. The function
is used in all the thinkers; the per-mode cohort effect is recorded as **not measured**, not
neutral, and belongs to the batch with the zoo's explicit arms (borg-zoo issue 54). This is the
first carry under the impact triage on consumer issue 426.

One structural note: the changed partial-damage branch is reached through
`borg_danger_one_kill(..., false)`. `borg_danger()` hardcodes `full_damage = true`
(`src/borg/borg-danger.c:2848`, pre-existing), so the direct call sites in
`src/borg/borg-fight-attack.c` (lines 1310 and 1332) are the ones that exercise it; other
`borg_danger(..., false)` callers do not, because the flag is overwritten before it is used.

Matrix group `hitchance-63` (2 cases) uses the `after-*.log` rows as its baseline.

## Ledger and credit

`patches/series.json` `HITCHANCE-63`, patch-active; consumer issue #437. Source defect, fix and
native fixture by Diego Amicabile (native proposal/evidence); carried unchanged.
