# source86: the left-ring duplicate check reads the stale right-slot entry (patch 0113)

Consumer adoption of [source 86](https://github.com/diegoami/neo-angband-tracker/issues/86)
(CBUG-0070), fix `aa3bf51c9` (source PR 488, evidence PR 487), kit
`docs/audit/c-reports/ringslot-86` at fixture commit `c4422e162`.

## Defect

`borg_best_stuff_aux()` searches for the best gear to wear and prevents the same physical item
from being chosen for both ring slots. Each slot's candidate is stored in `test[]` as the loop
goes. The right-ring branch compares against `test[n - 1]`, the left slot already chosen, which
is valid. The left-ring branch compared against `test[n + 1]`, the right slot, which has not been
chosen yet - it reads data left from a previous pass. The last ring tried on the right is
therefore never tried on the left, and a Borg that should wear two different rings can leave the
second slot unfilled.

## Fix

The left ring is chosen before the right, so only the right ring needs the duplicate check. Drop
the invalid left branch (2 insertions / 7 deletions). Source `aa3bf51c9` -> applied `5c34c5968`
(`cherry-pick -x`), patch `patches/0113-ringslot-left-duplicate.patch`, scope
`src/borg/borg-item-wear.c` only.

## Verification

Before commit `536534d2d`, tip `5c34c5968`, kit unchanged (`tools/verify-ringslot-86.py`).
Native gates green at both (965/965 unit, 5/5 frontend); byte-wise provenance over the source
diff; the `src`/`lib`/build scope is the one file.

| Mode | Before | After |
| --- | --- | --- |
| `swap` (new ring + kept ring) | **fails** 0/1: wears only the new ring (`wears_kept_ring=0`) | passes 1/1: wears both (`1`) |
| `mirror` (control) | passes: both worn | passes, **byte-identical** |
| `empty` (control: one ring only) | passes: one ring worn | passes, **byte-identical** |

The committed `before-*.log` / `after-*.log` and `result.json` are the product runs; the source
kit's own rows and file hashes are in `receipt.json`, carried unchanged.

## Mode attribution

Gear selection is shared code with no mode gate; it runs at the equipment steps in every
configuration (town and level changes), not per turn. The fixture shows the duplicate branch
letting the left slot try the second ring, while the mirror and one-ring controls keep their
decisions. Per-configuration cohort movement is recorded as **not measured**, not neutral, and
belongs to the batch with the zoo's explicit arms (borg-zoo issue 54).

Matrix group `ringslot-86` (3 cases) uses the `after-*.log` rows as its baseline. Full matrix
at this tree: 91 groups / 278 cases, `ok: true`, no failed group, 225/238 byte-identical; the
13 byte differences are in the same six groups as before this carry, so no new difference.

## Ledger and credit

`patches/series.json` `RINGSLOT-86`, patch-active; consumer issue #461. Source defect, fix and
native fixture by Diego Amicabile (native proposal/evidence); carried unchanged.
