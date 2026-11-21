# Recalldepth endgame answer carry: verified product scenario, ready for adoption

Consumer [216](https://github.com/diegoami/better-borg-c/issues/216),
canonical source
[310](https://github.com/diegoami/neo-angband-tracker/issues/310)
(recalldepth-310). Fix applied as
`b00f2a1901091b8a3222dc5576638684b36d95ea` (import of source PR373-head
`9609e2c8682c7f55fe1ae4bf9701ae74e2718ce0`, patch
`patches/0048-recalldepth-endgame-answer.patch`).

## Defect

`borg_recall()`'s endgame branch — `BI_MAXDEPTH == 99` with fewer than
five live uniques, cleaning up the last uniques to hit Morgoth —
answered the recall-depth reset prompt with `'n'` and noted
"# Not Resetting recall depth." while its own comment says "Do reset
Depth"; the generic branch below answers `'y'`. The corrected endgame
branch answers `'y'` ("# Resetting recall depth."), matching its
comment and the generic path. Behavior-affecting wrong-branch defect
(the endgame recall-depth reset never happened); no death or survival
claim.

## Pins and layered provenance

Before build: forty-seven-carry
`c61e90c9cdfb466336d3f564a1a3709f3daecc65` (retained tree
`../maimfoe`, normal and ASan cores, gate recorded at that commit,
source-identical to merged main `8a30a39d9` over
src/lib/CMakeLists.txt/cmake). `borg-escape.c` carries no earlier
corrections (upstream history only), so the verifier proves that the
source diff applied to the before module equals the product module;
the applied commit's diff body is byte-identical to the source commit's
diff. The source evidence fixture at `1f0425e12` is used unchanged; the
receipt chain is checked in the verifier against the run pin
`9ccd191c` (receipt.json sha256 map: scenario.c and borg.txt both
match; the evidence head adds only the receipt/replay artifacts on top
of the pin).

## Verified results (`artifacts/rd310/scenarios`, 2026-11-08)

- `endgame` (production decision with curdepth 65 = hunted-out
  maxdepth, 2 live uniques): normal-before 0/1,
  `acted=1 keys=6 reset_y=0 keep_n=1` — the prompt answered `'n'`
  (`<Escape> <Escape> r d n <Escape>`), failing the clear
  `require(!keep_n)`; normal-after 1/1, `reset_y=1 keep_n=0` —
  answered `'y'` (`<Escape> <Escape> r d y <Escape>`).
- `generic` (curdepth 30 = maxdepth): answered `'y'` both, 1/1,
  byte-identical before/after.
- `mismatch` (curdepth 65 ≠ hunted 30): answered `'n'` both, 1/1,
  byte-identical before/after.
- **Cross-repo byte match:** all six normal-phase log SHA256s are
  byte-identical to the source evidence kit's recorded logs
  (`before-endgame.log` `10885d84…`, `after-endgame.log` `339c02e5…`,
  generic `5c188cd1…`, mismatch `855bcdcc…`) — the consumer
  reproduction of the unchanged fixture matches the source run byte for
  byte.
- asan-before/asan-after mirror the normal phases; the sanitized runs
  report the pre-existing init-time `borg_sort_comp_hook` overflow
  (source finding #399, separately filed) in every phase — expected, and
  asserted as such rather than suppressed.
- `result.json` records `verified: true` with all commands and hashes.

Native gate on this core at the fix commit: **965/965 unit + 5/5
frontend** numerically read; tooling **95/95**. Regression matrix
(`artifacts/rd310-carry/regression.py`, local-only): all **151 prior
native fixture cases** match the recorded forty-seven-carry tally —
148 whole-log identical (including the gloves-205, detectcarry-219 and
maimfoe-256 cases), the dispel-248 undead case passing with its
asserted losdispel log change, and the two dispel-259 cases
asserted-red at the pinned post-losdispel values.

## Scope and limits

Bounded recall-decision scenario with one endgame case and two
controls. It proves the prompt answer and its correction, not a
gameplay frequency or survival claim. The correction makes the endgame
borg actually reset its recall depth (heading down for the last
uniques), so the measurement cohort is watched for divergence rather
than assumed neutral.

## Exact next work

1. PR from `task/carry-recalldepth`, agent evidence review plus
   required hosted checks, then normal merge.
2. On merged main: prepare the measurement build
   (`tools/measure-native.py prepare --observe`), run frozen v1 seeds
   11/29/47, cap 20000, wall 120, stall 30, and compare against the
   forty-seven-carry lineage run.
3. Mark 216 carried/open+patch-active; receipts on issue 216, source
   310 and checkpoint 1; evidence archive under `artifacts/rd310-carry/`.
4. Then dontreact-14 (patch 0049, source PR #405 green — 26 SUCCESS,
   evidence PR #406, five-site rename, four files).
