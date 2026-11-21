# Munchkin stair carry: verified product scenario, ready for adoption

Consumer [63](https://github.com/diegoami/better-borg-c/issues/63), canonical
source [244](https://github.com/diegoami/neo-angband-tracker/issues/244)
(munchkin-244). Fix applied as `2cf521b8bc8457eb0e585cb008077c484f7f0d3c`
(import of source PR301 `5b31e13d1023a64bf4f8613352e738757b5f56ec`, patch
`patches/0015-munchkin-stair-own-grid.patch`).

## Defect

The adjacent-monster scan in `borg_think_dungeon_munchkin()` reused the
pointer to the Borg's own grid for each neighbour, and the take-the-stair
tests after it read that stale neighbour's feature: standing on plain
floor beside a remembered down staircase — the grid the scan examines
last — the Borg queued the take-stair key (`'>'` recorded with
`own_is_stair=0`). In the game that keypress fails without taking a turn,
so the same decision can repeat indefinitely. The five-line correction
re-acquires the own grid after the scan, the pattern the power-climbing
section above already uses. This also documents why the missed-stair
direction of the report is shadowed in practice: legitimate stair-taking
is handled before the scan.

## Pins and provenance

Before build: fourteen-carry `25be9fab8ca17ddbb4a1101f063246c7605a1024`
(retained tree `../arc-angle`, normal and ASan cores, gate at `92003d66c`,
source-identical to `25be9fab8` over src/lib).
`borg-think-dungeon.c` is pristine in this product, so the imported module
is byte-identical to the reviewed source commit (module SHA256
`34fc4b9b292ccab203ef18382c47d2cfe2574188c579ace7e9539fb238f5a696`).
Profile preparation follows the evidence's own reproduce.py: the pinned
borg.txt ships the default and the runner switches
`borg_munchkin_start = FALSE` to `TRUE` exactly once. The first runner
attempt copied borg.txt verbatim and every phase failed identically at
setup; that tooling failure is retained under
`artifacts/m244/scenarios-setupfail` (no product behavior in it).

## Verified results (`artifacts/m244/scenarios`, 2026-11-06)

- normal-before `phantom`: 0/1, `acted=1 stair_key=62 own_is_stair=0` —
  the phantom take-stair keypress on plain floor.
- normal-after `phantom`: 1/1, `acted=1 stair_key=0 own_is_stair=0`.
- `on-stair` control: `acted=1 stair_key=62 own_is_stair=1`, 1/1 on every
  phase, byte-identical logs across all phases — legitimate stair-taking
  untouched.
- asan-before/after mirror the normal phases; no sanitizer report.
- `result.json` records `verified: true` with all commands and hashes.

Native gate on this core: **965/965 unit + 5/5 frontend**. Regression
matrix (`artifacts/m244-carry/regression.py`): all **74 prior native
fixture cases whole-log identical** to their baselines (60 nine-carry +
3 glyph + 3 missile + 3 tp-other + 3 dispel + 2 arc). Tooling 29/29.

## Scope and limits

Bounded native scenario: a direct call into the dungeon brain's munchkin
section with the configuration the game itself loads, the temp/target
state the flow uses, and a remembered staircase placed as the report
describes. No claim of an observed livelock in autonomous play — the
indefinite-repeat consequence is the game's documented no-turn keypress
failure mode, not a measured loop. Danger figures are the Borg's
heuristic.

## Exact next work

1. PR from `task/carry-munchkin-stair`, agent evidence review plus
   required hosted checks, then normal merge.
2. On merged main: frozen v1 measurement seeds 11/29/47, cap 20000, wall
   120, stall 30.
3. Mark 63 carried/open+patch-active; receipts on issue 63, source 244
   and checkpoint 1; evidence archive under `artifacts/m244-carry/`.
4. Then vampire-257 (PR293/294, one token in borg-fight-attack.c) — the
   remaining demonstrated pair — plus any newer source207 demonstrations.
