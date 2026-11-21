# source71: measure item pickups from the nearest up staircase (patch 0104)

Consumer adoption of [source 71](https://github.com/diegoami/neo-angband-tracker/issues/71)
(CBUG-0055), fix `97c424892` (source PR 494, evidence PR 493), kit
`docs/audit/c-reports/takestair-71` at fixture commit `13339abff`.

## Defect

A low-level Borg keeps item hunts within a leash of its escape staircase. The three pickup
flows `borg_flow_take()`, `borg_flow_take_scum()` and `borg_flow_take_lunal()` each scan the
known up staircases for the *nearest* one, but the scan kept the farthest:

```c
/* skip the closer ones */
if (b_j >= j)
    continue;
```

The leash tests that follow (`borg_flow_cost_stair()`, `borg_flow_far_from_stairs()`) then
measure every item against a distant staircase. In the source fixture a level-1 Warrior
standing two steps from an up staircase ignores a Flask of Oil three steps away in all three
flows once a second up staircase 55 steps away is known; with a single staircase the two
builds are identical.

## Fix

Keep the nearest known up staircase in the three scans (`if (b_j <= j && b_j != -1)
continue;`), matching the comments and the "closest stair" origin that `borg_flow_spread()`
expects. Source `97c424892` -> applied `595a7b9bd` (`cherry-pick -x`), patch
`patches/0104-takestair-nearest-stair.patch`, scope `src/borg/borg-flow-take.c` only.

## Verification

Before commit `32d3e6806`, tip `595a7b9bd`, product gate at `13339abff` (the kit commit; the
`src` tree is unchanged from the fix), kit unchanged (`tools/verify-takestair71.py`). Native
gates green at both (965/965 unit, 5/5 frontend); byte-wise provenance over the source diff;
the `src`/`lib`/build scope is the one file.

| Mode | Before | After |
| --- | --- | --- |
| `take` (`borg_flow_take(true, 250)`) | **fails** 0/1: the far staircase is the reference, the flask is over the leash, `goes_for_item=0` | passes 1/1: `goes_for_item=1`, key 6 east |
| `scum` (`borg_flow_take_scum(true, 6)`) | **fails** 0/1: `borg_flow_far_from_stairs()` measures from the far staircase | passes 1/1: `goes_for_item=1` |
| `lunal` (`borg_flow_take_lunal(true, 4)`) | **fails** 0/1: the flask is skipped as too far from the far staircase | passes 1/1: `goes_for_item=1` |
| `control` (single up staircase) | passes | passes, **byte-identical** (`e337ea33...`) |

The committed `before-*.log` / `after-*.log` and `result.json` are the product runs; the
source kit's own rows and file hashes are in `receipt.json`, carried unchanged.

## Mode attribution

The fixture drives each flow directly, so the changed branch is shown to execute in all three
flows - it is not a gate that makes two builds identical. Which runtime configurations reach
each flow in play is a call-site fact:

| Configuration | `borg_flow_take` | `borg_flow_take_scum` | `borg_flow_take_lunal` |
| --- | --- | --- | --- |
| unrestricted | reachable | reachable | reachable |
| munchkin / Lunal | reachable | reachable | reachable (only here, `borg_think_dungeon_lunal()` / `borg_think_dungeon_munchkin()`) |
| normal only | reachable | reachable (`borg_think_stair_scum()` at depth 1 while fleeing to town) | **unreachable by construction** (no Lunal/munchkin thinker) |

This is an unconditional change in the three shared pickup flows (no mode gate), so the
per-configuration cohort effect is recorded as **not measured**, not neutral, and belongs to
the batch measurement with the zoo's explicit arms (borg-zoo issue 54). What is measured here
is the function-level flip in every flow the change touches.

Matrix group `takestair-71` (4 cases) uses the `after-*.log` rows as its baseline.

## Ledger and credit

`patches/series.json` `TAKESTAIR-71`, patch-active; consumer issue #440. Source defect, fix and
native fixture by Diego Amicabile (native proposal/evidence); carried unchanged.
