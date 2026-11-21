# source60: find the first empty pack slot when the pack is empty (patch 0111)

Consumer adoption of [source 60](https://github.com/diegoami/neo-angband-tracker/issues/60)
(CBUG-0044), fix `adcb53459` (source PR 492, evidence PR 491), kit
`docs/audit/c-reports/emptyslot-60` at fixture commit `f31d1b5be`.

## Defect

`borg_first_empty_inventory_slot()` scans the pack backwards for the last used slot:

```c
for (i = PACK_SLOTS - 1; i >= 0; i--)
    if (borg_items[i].iqty) {
        if ((i + 1) < PACK_SLOTS)
            return i + 1;
        break;
    }
return -1;
```

When the pack is completely empty the loop runs to exhaustion without a break and falls through
to `return -1`, which the callers read as "no room". The shop food path uses this slot index, so
a starving Borg whose pack is empty is told it cannot buy food.

## Fix

Find the last used slot first (`i = -1` when the pack is empty), then return the slot after it if
the pack has room. Source `adcb53459` -> applied `0b3517c9d` (`cherry-pick -x`), patch
`patches/0111-emptyslot-empty-pack.patch`, scope `src/borg/borg-inventory.c` only.

## Verification

Before commit `230e6fc54`, tip `0b3517c9d`, kit unchanged (`tools/verify-emptyslot60.py`).
Native gates green at both (965/965 unit, 5/5 frontend); byte-wise provenance over the source
diff; the `src`/`lib`/build scope is the one file.

| Mode | Before | After |
| --- | --- | --- |
| `empty` (starving, empty pack, food shop) | **fails** 0/1: no food bought | passes 1/1: food bought |
| `oil` (one Flask of Oil in the pack) | passes, control: food bought | passes, **byte-identical** |

The committed `before-*.log` / `after-*.log` and `result.json` are the product runs; the source
kit's own rows and file hashes are in `receipt.json`, carried unchanged.

## Mode attribution

The slot helper is shared code with no mode gate: every shop purchase asks it for a slot, so the
changed return executes in unrestricted, munchkin/Lunal and normal play alike. The behaviour
difference only appears when the pack is empty and something is being bought - a starvation-path
event, not a per-turn one. The fixture shows the empty-pack branch executing and the non-empty
control unchanged. Per-configuration cohort frequency is recorded as **not measured**, not
neutral, and belongs to the batch with the zoo's explicit arms (borg-zoo issue 54).

The empty-pack return is `0`, not `-1`, which is a slightly wider change than the starvation path
alone: `borg_inventory_full()` tests `== -1`, and the buy, wear and junk callers use the value as
a pack index. Every caller handles `0` correctly (slot 0 is a valid empty slot when the pack is
empty), and the non-empty branches are unchanged.

Matrix group `emptyslot-60` (2 cases) uses the `after-*.log` rows as its baseline. Full matrix
at this tree: 89 groups / 273 cases, `ok: true`, no failed group, 220/233 byte-identical; the
13 byte differences are in the same six groups as before this carry, so no new difference.

## Ledger and credit

`patches/series.json` `EMPTYSLOT-60`, patch-active; consumer issue #457. Source defect, fix and
native fixture by Diego Amicabile (native proposal/evidence); carried unchanged.
