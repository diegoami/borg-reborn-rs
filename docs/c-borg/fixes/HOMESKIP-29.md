# source29: reset the home-grab skip flag for each ware (patch 0115)

Consumer adoption of [source 29](https://github.com/diegoami/neo-angband-tracker/issues/29)
(CBUG-0010), fix `3f35efd71` (source PR 575, evidence PR 574), kit
`docs/audit/c-reports/homeskip-29` at fixture commit `83f648f70`.

## Defect

`borg_think_home_grab_useless()` walks the wares at Home and skips any item the Borg has just
sold there, so it does not immediately take it back. The `skip_it` flag was declared once outside
the item loop and only ever set true:

```c
if (sold_item_tval[p] == item->tval && sold_item_sval[p] == item->sval)
    skip_it = true;
```

Once one sold item matches, every later ware in the same pass is skipped, so useful wares are
left at Home.

## Fix

Reset `skip_it = false` at the top of each ware's check. Source `3f35efd71` -> applied
`ed7b2d953` (`cherry-pick -x`), patch `patches/0115-homeskip-per-ware.patch`, scope
`src/borg/borg-store-buy.c` only.

## Verification

Before commit `b77f5ae00`, tip `ed7b2d953`, kit unchanged (`tools/verify-homeskip29.py`). Native
gates green at both (965/965 unit, 5/5 frontend); byte-wise provenance over the source diff; the
`src`/`lib`/build scope is the one file.

| Mode | Before | After |
| --- | --- | --- |
| `stored-first` (sold item earlier in the list, useful Scroll of Light later) | **fails** 0/1: scroll not grabbed, pack empty | passes 1/1: scroll grabbed |
| `stored-last` (control: sold item last) | passes: scroll grabbed | passes, **byte-identical** |
| `no-history` (control: nothing sold) | passes: scroll grabbed | passes, **byte-identical** |

The committed `before-*.log` / `after-*.log` and `result.json` are the product runs; the source
kit's own rows and file hashes are in `receipt.json`, carried unchanged.

## Mode attribution

Town/home ware selection, shared code with no mode gate, run at every Home visit in every
configuration. The fixture shows the leak when a sold item precedes a useful ware. The
per-configuration cohort effect (and whether it is mostly visible in munchkin arms) is recorded
as **not measured**, not neutral, and belongs to the batch with the zoo's explicit arms
(borg-zoo issue 54).

Matrix group `homeskip-29` (3 cases) uses the `after-*.log` rows as its baseline. Full matrix
at this tree: 93 groups / 284 cases, `ok: true`, no failed group, 231/244 byte-identical; the
13 byte differences are in the same six groups as before this carry, so no new difference.

## Ledger and credit

`patches/series.json` `HOMESKIP-29`, patch-active; consumer issue #464. Source defect, fix and
native fixture by Diego Amicabile (native proposal/evidence); carried unchanged.
