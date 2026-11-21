# Panelrows-348 carry: verified product scenario (memory safety)

Consumer [373](https://github.com/diegoami/better-borg-c/issues/373),
canonical source
[348](https://github.com/diegoami/neo-angband-tracker/issues/348)
(panelrows-348; source fix `82702575b`, evidence fixture `a9303c7ca`).
Applied as a `cherry-pick -x` of source
`82702575b2179cb27cb7eac24272515dfea0830e`, patch
`patches/0097-panel-scan-bounds.patch` (applied commit recorded below).

## Defect

`borg_flow_light()`, `borg_flow_vault()` and `borg_excavate_vault()`
scan the visible map panel (`w_y .. w_y + SCREEN_HGT` ×
`w_x .. w_x + SCREEN_WID`) and index `borg_grids[y][x]` for each grid.
`borg_grids` has `DUNGEON_HGT` (66) rows of `DUNGEON_WID` (198) grids,
but the panel grows with the terminal: with the default sidebar, a
terminal of 69 or more rows or 213 or more columns shows more rows or
columns than a full-size level has. The game then places the panel at
the level's edge and the scans read past the last row pointer of
`borg_grids` (taller terminal) or past the end of a row (wider
terminal). With a 70-row terminal the read of the missing row **crashes
the game**; a wider terminal silently reads neighbouring memory. The
correction stops each scan at the edge of the Borg's map
(`AUTO_MAX_Y`/`AUTO_MAX_X`); the grids that are on the map are scanned
exactly as before.

## Memory-safety verification

Per the verification contract (0068/0070 precedent), this carry adds
AddressSanitizer phases: four cores — normal and ASan (`-fsanitize=address
-g -DNDEBUG`, Release, exactly like the source kit) at the before commit
and at the tip. Expected: before, each tall-terminal mode crashes with a
global-buffer-overflow READ of size 8 (the `borg_grids` row pointer) from
its own scan function, and each wide-terminal mode reports a
heap-buffer-overflow READ of size 2 (the grid byte); after, every mode
passes report-free in both builds, and the seven wide/normal/fit
controls are byte-identical across the correction in both builds.

## Pins and provenance

Before: merged main `9a428ea38` (snapshotted build `build/native-before`,
gate result `artifacts/native-before/result.json`, 965/965 + 5/5 at that
commit; ASan twin `build/asan-before`). Both modules are otherwise
pristine on the product. The verifier proves that the source diff
applied to the before modules equals the product modules and that the
applied commit is the only `src`/`lib` difference from the before
commit. Fixture: kit `a9303c7ca` at head `24159739b`, asserted
byte-identical to the run pin.

## Verified results (`artifacts/pr348/scenarios`, 2026-11-09)

- Normal build, before: `light-tall`, `vault-tall`, `dig-tall` crash
  with SIGSEGV (exit -11, no outcome row — the read of the missing row
  pointer); the seven wide/normal/fit controls complete cleanly with
  the kit's recorded rows.
- Normal build, after: all ten modes pass with the kit's recorded rows
  (`targets` 88/66/1 as recorded); the seven controls are
  **byte-identical** to their before logs (`1a338049`-style identities
  recorded in `result.json`) — the on-map grids are scanned exactly as
  before.
- ASan build, before: all six out-of-map modes (tall **and** wide)
  report a sanitizer READ from their own scan function — the tall
  modes as a **global-buffer-overflow READ of size 8** (the
  `borg_grids` row pointer past the last row), the wide modes as a
  **heap-buffer-overflow READ of size 2** (the grid byte past a heap
  region); sites asserted per mode (`borg_flow_light`,
  `borg_flow_vault`, `borg_excavate_vault`). The normal/fit controls
  complete report-free.
- ASan build, after: every mode passes report-free; the controls are
  byte-identical to their asan-before logs.
- `result.json` records `verified: true` with all commands, pins and
  hashes. Harness notes recorded in the verifier commit: the
  ASan-before core is built in a separate worktree frozen at the
  before commit (an in-tree rebuild after the cherry-pick had silently
  recompiled the changed objects with the fixed source); the kit's
  missing `obj-tval.h` include is not needed here, but the
  `OurUnitTestLib` target must be built explicitly in ASan trees (the
  default target skips it); the OOB detection distinguishes the
  global-buffer-overflow (tall) and heap-buffer-overflow (wide) kinds
  of the same defect.

## Scope and limits

Ten panel-scan scenarios (light/vault/dig × tall/wide/normal/fit
terminals). They prove the scan bounds and the crash fix; no survival
claim.

## Exact next work

1. PR from `task/carry-panelrows`, agent evidence review plus required
   hosted checks, then normal merge.
2. Frozen v1 cohort at the merged tip against the tporod tip cohort.
3. Receipts on issue 373, source 348 and checkpoint 1.
