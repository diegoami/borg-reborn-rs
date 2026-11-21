# source31: restore the pack entry after home swap trials (patch 0114)

Consumer adoption of [source 31](https://github.com/diegoami/neo-angband-tracker/issues/31)
(CBUG-0012), fix `571380b3d` (source PR 581, evidence PR 580), kit
`docs/audit/c-reports/swaptrial-31` at fixture commit `e1a3a7d44`.

## Defect

The home buy/swap weapon and armour routines score a candidate by taking it out of the pack into
a temporary hole slot, after saving the pack record in `safe_items`. Both restore paths wrote

```c
/* Restore hole */
borg_items[hole].iqty = 0;
```

instead of restoring the saved entry. When the trial's hole is the Borg's current swap slot, the
carried backup weapon or armour is erased from the pack record: the model thinks the slot is
empty while the game still holds the item. The Borg then shops for a replacement it already
carries, or stores/retrieves the wrong item (in the fixture, the Evenstar is retrieved twice and
stored once).

## Fix

Restore `memcpy(&borg_items[hole], &safe_items[hole], sizeof(borg_item))` in both routines.
Source `571380b3d` -> applied `5e248e8a4` (`cherry-pick -x`), patch
`patches/0114-swaptrial-restore-hole.patch`, scope `src/borg/borg-store-buy.c` only.

## Verification

Before commit `4a58046f8`, tip `5e248e8a4`, kit unchanged (`tools/verify-swaptrial31.py`). Native
gates green at both (965/965 unit, 5/5 frontend); byte-wise provenance over the source diff; the
`src`/`lib`/build scope is the one file.

| Mode | Before | After |
| --- | --- | --- |
| `swap-held` (swap weapon in the trial hole) | **fails** 0/1: model does not match the game; the Borg fetches the Evenstar it already holds (`evenstar_retrieved=1`) | passes 1/1: model matches, no retrieval or store |
| `no-weapon-swap` (control) | passes | passes, **byte-identical** |
| `no-candidate` (control) | passes | passes, **byte-identical** |

The committed `before-*.log` / `after-*.log` and `result.json` are the product runs; the source
kit's own rows and file hashes are in `receipt.json`, carried unchanged.

**Disclosed divergence from the source receipt:** the source lane recorded the before phase as
`evenstar_retrieved=2, evenstar_stored=1` (stash, then fetch); on this product tree the before
phase fetches once (`1/0`). The swap valuation differs (`weapon_swap_value` 113399 here against
132749 on the source base, which carries none of our 110 prior patches), so the wrong path is
shorter. The kit's own pass criterion still flips (model does not match -> matches) and the after
rows are identical to the source receipt; only the before-phase count is adapted.

## Mode attribution

Town/home shop and equipment logic, shared code with no mode gate, run at shop and home visits in
every configuration. The triage notes this class may look inert in a normal arm and be active in
a munchkin arm, so the per-configuration effect is recorded as **not measured**, not neutral, and
belongs to the batch with the zoo's explicit arms (borg-zoo issue 54).

Matrix group `swaptrial-31` (3 cases) uses the `after-*.log` rows as its baseline. Full matrix
at this tree: 92 groups / 281 cases, `ok: true`, no failed group, 228/241 byte-identical; the
13 byte differences are in the same six groups as before this carry, so no new difference.

## Ledger and credit

`patches/series.json` `SWAPTRIAL-31`, patch-active; consumer issue #462. Source defect, fix and
native fixture by Diego Amicabile (native proposal/evidence); carried unchanged.
