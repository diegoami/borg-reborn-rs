# source80: only drink cure potions for food when actually hurt (patch 0112)

Consumer adoption of [source 80](https://github.com/diegoami/neo-angband-tracker/issues/80)
(CBUG-0064), fix `8ae589ac5` (source PR 467, evidence PR 466), kit
`docs/audit/c-reports/eathurt-80` at fixture commit `326fe7698`.

## Defect

`borg_eat_food_any()` tries, in order, to eat food; potions of healing are used only "when
hurting":

```c
if ((borg.trait[BI_CURHP] < 4
        || (borg.trait[BI_CURHP] <= borg.trait[BI_MAXHP]))
    && (borg_quaff_potion(sv_potion_cure_light)
        || borg_quaff_potion(sv_potion_cure_serious)
        || borg_quaff_potion(sv_potion_cure_critical)
```

`BI_CURHP <= BI_MAXHP` is always true, so the gate never applies: a weak Borg at full health
drinks a Cure potion before casting Remove Hunger, spending healing it does not need.

## Fix

Test `BI_CURHP < BI_MAXHP`. Source `8ae589ac5` -> applied `9791d3361` (`cherry-pick -x`), patch
`patches/0112-eathurt-when-hurt.patch`, scope `src/borg/borg-item-use.c` only.

## Verification

Before commit `3e7732edd`, tip `9791d3361`, kit unchanged (`tools/verify-eathurt80.py`). Native
gates green at both (965/965 unit, 5/5 frontend); byte-wise provenance over the source diff; the
`src`/`lib`/build scope is the one file.

| Mode | Before | After |
| --- | --- | --- |
| `fullhp` (weak, 69/69 HP, 2 cures, Remove Hunger known) | **fails** 0/1: cure quaffed, no spell cast | passes 1/1: Remove Hunger cast, no potion |
| `hurt` (weak, 64/69 HP) | passes, control: cure quaffed | passes, **byte-identical** |

The committed `before-*.log` / `after-*.log` and `result.json` are the product runs; the source
kit's own rows and file hashes are in `receipt.json`, carried unchanged.

## Mode attribution

The consumption routine is shared code with no mode gate and runs whenever the Borg wants food, in
every configuration. The difference appears only while weak (hunger) and at full health, which is
common in play; the fixture shows the full-health branch flipping while the hurt branch keeps the
potion. Per-configuration cohort movement is recorded as **not measured**, not neutral, and
belongs to the batch with the zoo's explicit arms (borg-zoo issue 54).

Matrix group `eathurt-80` (2 cases) uses the `after-*.log` rows as its baseline. Full matrix
at this tree: 90 groups / 275 cases, `ok: true`, no failed group, 222/235 byte-identical; the
13 byte differences are in the same six groups as before this carry, so no new difference.

## Ledger and credit

`patches/series.json` `EATHURT-80`, patch-active; consumer issue #459. Source defect, fix and
native fixture by Diego Amicabile (native proposal/evidence); carried unchanged.
