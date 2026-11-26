# Stair-track carry: verified product scenario, ready for adoption

Consumer [137](https://github.com/diegoami/better-borg-c/issues/137), canonical
source [232](https://github.com/diegoami/neo-angband-tracker/issues/232)
(stairtrack-232, priority low). Fix applied as
`faa86f83b12f7f43cdde21f48faf045f9e385c60` (import of source PR358
`8310e9cc9fe90a82f93609f542d5f057b835fbcf`, patch
`patches/0036-stair-track-break.patch`).

## Defect

`borg_think_stair_scum()`'s up-staircase "already known" scan used
`continue` where the matching down-staircase scan uses `break`: a match no
longer stopped the loop, and the staircase under the Borg — already tracked,
for example from being seen on the map earlier — was appended to `track_less`
again on every visit (1 grows to 2 per visit, until the array fills and later
staircases can no longer be tracked). The one-keyword correction breaks on
the match. The measured fact is the array growth; the failure mode is an
untracked staircase, not a crash; no death or survival claim.

## Pins and layered provenance

Before build: thirty-five-carry `224c0d81f4aa37b4a54c868b1a248e092d280bf6`
(retained tree `../brand-need`, normal and ASan cores, gate recorded at that
commit, source-identical to merged main `638513e91` over
src/lib/CMakeLists.txt/cmake). `borg-think-dungeon-util.c` already carries
0032 — the lunal bail inside this same function — so the verifier proves
that the source diff applied to the before module equals the product module,
and the applied commit's diff is byte-identical to the source commit's diff.
The source evidence fixture at `07a71c16` (evidence PR359) is used unchanged.

## Verified results (`artifacts/s232/scenarios`, 2026-11-07)

- normal-before `known` (staircase pre-tracked, then the production
  discovery path runs): 0/1, `before=1 after=2` — the known staircase is
  re-added as a duplicate.
- normal-after `known`: 1/1, `before=1 after=1` — breaking on the match
  leaves the array alone.
- `fresh` (never-tracked staircase): `before=0 after=1`, 1/1 on every
  phase, byte-identical before/after.
- asan-before/asan-after mirror the normal phases; no sanitizer report.
- `result.json` records `verified: true` with all commands and hashes.

Native gate on this core at the fix commit: **965/965 unit + 5/5 frontend**
numerically read; tooling **41/41**. Regression matrix
(`artifacts/s232-carry/regression.py`, local-only): all **117 prior native
fixture cases** match the recorded thirty-five-carry tally — 114 whole-log
identical (including the two brand-240 cases), the dispel-248 undead case
passing with its asserted losdispel log change, and the two dispel-259
cases asserted-red at the pinned post-losdispel values.

## Scope and limits

Bounded native production stair-scum discovery scenario with a pre-tracked
and a fresh staircase. It proves the duplicate-tracking behavior and its
correction, not a gameplay frequency or survival claim.

## Exact next work

1. PR from `task/carry-stairtrack`, agent evidence review plus required
   hosted checks, then normal merge.
2. On merged main: prepare the measurement build
   (`tools/measure-native.py prepare --observe`), run frozen v1 seeds
   11/29/47, cap 20000, wall 120, stall 30, and compare outcomes and trace
   hashes with the recorded thirty-four-carry run (unchanged through the
   thirty-fifth carry).
3. Mark 137 carried/open+patch-active; receipts on issue 137, source 232
   and checkpoint 1; evidence archive under `artifacts/s232-carry/`.
4. Recheck source207; resume the recorded order (source348 panel-vs-`DUNGEON_WID`
   overread lead is source-owned and was not consumer-ready; intent items
   follow the source lane) unless a newer demonstration becomes ready.
