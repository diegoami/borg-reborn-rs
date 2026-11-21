# source133: the unidentified-ego sell check asks about the worn cloak (patch 0117)

Consumer adoption of [source 133](https://github.com/diegoami/neo-angband-tracker/issues/133)
(CBUG-0117), fix `8b5ab27a8` (source PR 496, evidence PR 495), kit
`docs/audit/c-reports/egosell-133` at fixture commit `3a95377ad`.

## Defect

`borg_good_sell()` refuses to sell a single unidentified item whose ego has a random power, since
the power cannot be known yet. The check read the worn cloak's ego instead of the item's:

```c
if (!item->ident && item->ego_idx && item->iqty < 2) {
    if (borg_ego_has_random_power(
            &e_info[borg_items[INVEN_OUTER].ego_idx])) {
        return false;
    }
}
```

With an ordinary cloak worn, an unidentified Cloak of Aman in the pack is sold (a blessed item
lost); with a random-power cloak worn, items that are safe to sell are refused.

## Fix

Test `item->ego_idx`. Source `8b5ab27a8` -> applied `927b51ff4` (`cherry-pick -x`), patch
`patches/0117-egosell-item-ego.patch`, scope `src/borg/borg-store-sell.c` only.

## Verification

Before commit `858774476`, tip `927b51ff4`, kit unchanged (`tools/verify-egosell133.py`). Native
gates green at both (965/965 unit, 5/5 frontend); byte-wise provenance over the source diff; the
`src`/`lib`/build scope is the one file.

| Mode | Before | After |
| --- | --- | --- |
| `plain-worn` (ordinary cloak worn, unidentified Cloak of Aman in pack) | **fails** 0/1: the Cloak of Aman is sold | passes 1/1: kept |
| `aman-worn` (control: the Cloak of Aman worn) | passes | passes, **byte-identical** |

The committed `before-*.log` / `after-*.log` and `result.json` are the product runs; the source
kit's own rows and file hashes are in `receipt.json`, carried unchanged.

## Mode attribution

Town/sell logic, shared code with no mode gate, run at every shop sell pass in every
configuration. The fixture shows the guard now reading the sold item's ego. The
per-configuration cohort effect is recorded as **not measured**, not neutral, and belongs to the
batch with the zoo's explicit arms (borg-zoo issue 54).

Matrix group `egosell-133` (2 cases) uses the `after-*.log` rows as its baseline. Full matrix
at this tree: 95 groups / 290 cases, `ok: true`, no failed group, 237/250 byte-identical; the
13 byte differences are in the same six groups as before this carry, so no new difference.

## Ledger and credit

`patches/series.json` `EGOSELL-133`, patch-active; consumer issue #468. Source defect, fix and
native fixture by Diego Amicabile (native proposal/evidence); carried unchanged.
