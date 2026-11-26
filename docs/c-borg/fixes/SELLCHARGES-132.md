# source132: restore staff and wand charges after home-sell trials (patch 0116)

Consumer adoption of [source 132](https://github.com/diegoami/neo-angband-tracker/issues/132)
(CBUG-0116), fix `58a2638c7` (source PR 460, evidence PR 459), kit
`docs/audit/c-reports/sellcharges-132` at fixture commit `14012c86e`.

## Defect

The home-sell trial takes a stackable item out of the pack to score it, keeping the per-item
charge count in `charge_each`. The restore path put the quantity back but not the charges:

```c
/* put item back into pack */
item->iqty++;

/* put the item back in the test array */
test_item[n] = n;
```

A wand or staff therefore loses its charges in the Borg's pack record after the trial - the model
sees an empty device. The Borg then takes the item out of its Home instead of storing a charged
device, or under-values what it carries.

## Fix

Add `charge_each` back to `item->pval` for `TV_STAFF` and `TV_WAND` in the restore path. Source
`58a2638c7` -> applied `e5e5bf4a1` (`cherry-pick -x`), patch
`patches/0116-sellcharges-restore.patch`, scope `src/borg/borg-store-sell.c` only.

## Verification

Before commit `2d0840571`, tip `e5e5bf4a1`, kit unchanged (`tools/verify-sellcharges132.py`).
Native gates green at both (965/965 unit, 5/5 frontend); byte-wise provenance over the source
diff; the `src`/`lib`/build scope is the one file.

| Mode | Before | After |
| --- | --- | --- |
| `stack` (staff stack at Home, charges counted in the trial) | **fails** 0/1: takes the Recall out, does not store the device | passes 1/1: stores the device, does not take the Recall |
| `stack-emptyhome` (control) | passes | passes, **byte-identical** |
| `single` (control: single staff) | passes | passes, **byte-identical** |
| `none` (control) | passes | passes, **byte-identical** |

The committed `before-*.log` / `after-*.log` and `result.json` are the product runs; the source
kit's own rows and file hashes are in `receipt.json`, carried unchanged.

## Mode attribution

Town/home sell logic, shared code with no mode gate, run at every Home sell pass in every
configuration. The fixture shows the charge loss making the stored device look empty. The
per-configuration cohort effect (and whether it is mostly visible in munchkin arms) is recorded
as **not measured**, not neutral, and belongs to the batch with the zoo's explicit arms
(borg-zoo issue 54).

Matrix group `sellcharges-132` (4 cases) uses the `after-*.log` rows as its baseline. Full matrix
at this tree: 94 groups / 288 cases, `ok: true`, no failed group, 235/248 byte-identical; the
13 byte differences are in the same six groups as before this carry, so no new difference.

## Ledger and credit

`patches/series.json` `SELLCHARGES-132`, patch-active; consumer issue #466. Source defect, fix
and native fixture by Diego Amicabile (native proposal/evidence); carried unchanged.
