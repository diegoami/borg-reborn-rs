# Glowtrace permanent-glow correction carry: verified product scenario

Consumer [224](https://github.com/diegoami/better-borg-c/issues/224),
canonical source
[213](https://github.com/diegoami/neo-angband-tracker/issues/213)
(glowtrace-213, revised kit). Correction applied as `b5e07827a`
(`cherry-pick -x` of source PR411 head
`c316a65b248c4372ff3b686eafaa0477c3117eeb`, patch
`patches/0069-glow-floor-permanent-glow-flag.patch`), stacked on the carried
0050 shape (`da9934216`, `patches/0050-glowtrace-floors-branch.patch`).

## Defect

The carried glowtrace fix restored the map update's dead Floors branch but
read the glow state through `square_islit()` — the grid's current light
level, which includes the Borg's own light source. Every floor inside the
torch radius was therefore marked `BORG_GLOW`, and the flow-toward-light
consumer chased targets the Borg was already lighting itself (the source
kit's torchlit diagnostic measured 9 such targets from the Borg's own
torch). The owner decision on source 213 chose `square_isglow()`: the
revised kit (fixtures `8934edd4e`, head `f2b402c92`, superseding the
0050-era fixtures) applies the 3-line condition correction with the
torch-lit branch kept — a floor is glowing only when the game flags
`SQUARE_GLOW`, torch-lit floors take `BORG_LIGHT` instead, and the dark test
is `!isglow && !islit`.

## Pins and provenance

Before: merged main `72dd1c8a366a924b3f2f90c3a966c8aa6d6d75fa` (the tree
with patch 0068; snapshotted build `build/native-before`, gate result
`artifacts/native-before/result.json`, 965/965 + 5/5 at that commit). The
verifier proves that the source correction applied to the before
`borg-update.c` equals the product module and that the applied commit is the
only `src`/`lib` difference from the before commit. Product gate at the tip
(gate result `artifacts/native/result.json`). The fixture is the revised kit
at head `f2b402c92`, asserted byte-identical to the run pin `8934edd4e`.
Normal phases only (the correction touches no memory-safety behavior; same
scope as the most recent carries).

## Verified results (`artifacts/gtisg/scenarios`, 2026-11-08)

- `torchlit` — the fail-before/pass-after case: before 0/1 with the
  torch-lit floor marked `BORG_GLOW` from the Borg's own torch
  (`torchlit_marked_glow=1`, `near_glow=1`, the pre-existing `BORG_LIGHT`
  marking also set) and `glow_n=9`; after 1/1 with
  `torchlit_marked_glow=0`, `glow_n=0`, `near_glow=0`, `BORG_LIGHT` kept.
- `main` (glowing arena): 1/1 in both phases, logs byte-identical
  (`699d45d7…`), `distant_glow=1 near_glow=1 wall_glow=0`, marked/lit/pcmem
  1/1/1, flow-toward-light `glow_n=484` in both — the restored-branch
  contract is unchanged for genuinely glowing floors, value for value with
  the source kit.
- `door` control: 1/1 in both, log byte-identical (`2620cdb7…`).
- `unlit`: 1/1 in both, log byte-identical (`91acbece…`), the far floor
  `BORG_DARK` in both.
- `result.json` records `verified: true` with all commands, pins and
  hashes.

Native gate on the product: **965/965 unit + 5/5 frontend** (before and
product); tooling **129/129**. Prior-fixture matrix not rerun (uncommitted
runner; known gap recorded on checkpoint 1 and the milestone 6 review).

## Harness correction disclosed

The first verifier run asserted the before row's `near_light_flag` as 0;
the product's pre-existing lit marking sets `BORG_LIGHT` on that grid
independently of the corrected branch (the 0050 shape never set it there).
The measured before row is `near_glow=1` with `near_light_flag=1` and 9
flow targets; recorded in a verifier commit with the defect requirements
unchanged and stricter (the case must still fail, `near_glow` must be 1,
`glow_n` pinned to the kit's 9).

## Scope and limits

Bounded map-update scenario with one glowing-arena case, a torch-lit arena,
an unlit far floor and a door/wall control. It proves the glow-test
correction and its consumers, not a gameplay frequency or survival claim.

## Exact next work

1. PR from `task/carry-glow-isglow`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: frozen v1 cohort seeds 11/29/47 against a fresh
   before-cohort at `72dd1c8a3`; the torch-lit marking change is
   behavior-active near the Borg's own light, so any divergence is recorded
   as observed.
3. Receipts on issue 224, source 213 and checkpoint 1; evidence under
   `artifacts/gtisg/`.
4. Recheck the source board for the newer draft pairs (painmsg-197,
   spasticscore-147, trollswap-201, iswarded-198, drophole-136).
