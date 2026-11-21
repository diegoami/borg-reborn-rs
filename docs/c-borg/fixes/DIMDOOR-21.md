# Dimdoor-21 carry: verified product scenario

Consumer [335](https://github.com/diegoami/better-borg-c/issues/335),
canonical source
[21](https://github.com/diegoami/neo-angband-tracker/issues/21)
(dimdoor-21; source fix `4b7a9f414`, evidence fixture `3edecf91b`).
Applied as the first step of the dim-urg PR (`cherry-pick -x` of source
`4b7a9f414bec7b23aa3453dfb7e54ba2e85c871b`, patch
`patches/0086-dimension-door-known-grid.patch`), stacked with the
urgentshop-23 fix (patch 0087).

## Defect

`borg_dimension_door()` scored every in-bounds grid in a 100x100 box
around the borg and aimed at the lowest-danger one without checking the
player could land there. The scan starts in the far north-west corner,
so on a normal level the winner is usually a granite grid the borg has
never seen. The game does not move the player onto rock: `TELEPORT_TO`
searches outward from the target for the nearest legal grid, which can
be back in the room the borg was escaping from. The correction skips
grids not known to be floor, lava, and grids with a known monster, so
the chosen grid is one the game accepts and the scored danger is the
danger where the borg lands.

## Pins and provenance

Before: merged main `8b549c2f9` (snapshotted build `build/native-before`,
gate result `artifacts/native-before/result.json`, 965/965 + 5/5 at that
commit). `borg-escape.c` is otherwise pristine on the product. The
verifier proves that the source diff applied to the before module equals
the product module, that the fix commit's own scope is `borg-escape.c`
alone, and that the tree-level scope from before to the PR tip is this
file plus urgentshop-23's two files. Fixture: kit `3edecf91b` at head
`f8b7d662b`, asserted byte-identical to the run pin.

## Verified results (`artifacts/dd21/scenarios`, 2026-11-09)

- `granite` — fail before, pass after: before the borg aimed at an
  unseen grid (16,1) and the game landed it at (60,31), 59 grids away,
  back in the same room and in the feared region (`on_aim=0`,
  `same_room=1`, `fear_region_at_landing=78`); after it aims at a known
  floor grid (100,25) and lands on the aim (`on_aim=1`,
  `aim_distance=0`, feared region 0, no line of sight to the dread).
- `open` control: 1/1 in both phases, log byte-identical before/after
  (`588747b6…`).
- `result.json` records `verified: true` with all commands, pins and
  hashes. The verifier passed on its first complete run.

## Scope and limits

Bounded escape-cast scenarios (a granite-adjacent and an open-floor
position). They prove the aim rule and that the borg lands on the aimed
grid out of the feared region; no survival claim.

## Exact next work

1. Step 2 (urgentshop-23) in the same PR with a per-step gate; agent
   evidence review plus required hosted checks, then normal merge.
2. Frozen v1 cohort at the merged tip against the hurtstun tip cohort.
3. Receipts on issue 335, source 21 and checkpoint 1.
