# Reverse-flow carries (sources 146, 345): verified product scenarios

Two one-line corrections that run a reverse flow unoptimized, in two different files, adopted as
consecutive patches in one PR with a before-build snapshot at each step. They share one cause: an optimized
reverse flow starts at the Borg's own grid (cost 0) and stops before reaching any other grid.

| patch | consumer | source | file | applied | source commit |
| --- | --- | --- | --- | --- | --- |
| 0064 | [268](https://github.com/diegoami/better-borg-c/issues/268) | [146](https://github.com/diegoami/neo-angband-tracker/issues/146) spastic-146 | `borg-flow-misc.c` | `dafc4bd45c5cc45fa0a435ced1a956512fbdcafc` | `f97311e5d52de4dda8d75058114c52acbdac2674` |
| 0065 | [269](https://github.com/diegoami/better-borg-c/issues/269) | [345](https://github.com/diegoami/neo-angband-tracker/issues/345) glyphflow-345 | `borg-flow-glyph.c` | `e66b1dca298523c84a8a7a0d9ae009ddf6562b10` | `f618381294dcf5bff1f0bc95f0fbe259412a53b9` |

Each is a `cherry-pick -x` of one reviewed C-only source commit.

## Defects

* **146 (spastic search):** the spastic-search reverse flow was optimized, so it never produced a move (`moved` 0).
  It now runs unoptimized.
* **345 (Sea of Runes):** the optimized reverse flow in the glyph selection stops at an existing centre rune and never
  selects outlying runes (`progress` 0). It now runs unoptimized. Reachability stated on the source issue: its only
  caller runs when Morgoth is near and the Borg is not already in position (see the source issue for the full
  condition), so this is a late-game path.

Behaviour-active where those flows run. No survival claim.

## Pins and provenance

Before: `origin/main` `afdd7921d8b945bcc0efe50972b5ac2f04fb5897`. Evidence fixtures unchanged (heads `33f288ef664527ddef80dcda928a8e8db6b1cff5`,
`4695d78e4e6ed5e2f0361520029c86badfa6c2ba`; runtime files byte-identical to the run pins `da9a39f4d`,
`fbe06401c`, asserted). The two patches touch different files, so the plain source diff applies
to each before module and the product module equals before + source diff; each verifier asserts that its file is the
only `src`/`lib`/build-file difference from its before commit.

## Verification (`artifacts/spastic`, `artifacts/glyphflow`; 2026-11-08)

```sh
git remote add source-tracker https://github.com/diegoami/neo-angband-tracker.git
git fetch source-tracker task/spastic-146 task/spastic-146-evidence task/glyphflow-345 task/glyphflow-345-evidence
# build origin/main, snapshot build/native (SNAP0); build the 0064 commit, snapshot (SNAP1); build the tip, then
flock /tmp/angband-validation.lock python3 tools/verify-spastic.py --before-build SNAP0 --fix dafc4bd45c5cc45fa0a435ced1a956512fbdcafc --output artifacts/spastic/scenarios
flock /tmp/angband-validation.lock python3 tools/verify-glyphflow.py --before-build SNAP1 --fix e66b1dca298523c84a8a7a0d9ae009ddf6562b10 --output artifacts/glyphflow/scenarios
flock /tmp/angband-validation.lock python3 tools/verify-native.py --jobs 10   # on the tip
```

| source | mode | before | after |
| --- | --- | --- | --- |
| 146 | `middle` | 0/1, `moved=0` | **1/1**, `moved=1` |
| 146 | `corner` (own grid searched first) | 0/1, `moved=0` | **1/1**, `moved=1` |
| 146 | `far` | 1/1, `moved=0` | same; log byte-identical |
| 345 | `center` (existing centre rune) | 0/1, `progress=0` | **1/1**, `progress=1` |
| 345 | `offcenter` | 1/1, `progress=1` | same; log byte-identical |

All rows equal the source evidence's recorded rows value for value. Native gate on the tip
`e66b1dca2`: **965/965 unit + 5/5 frontend**; tooling 121 pass. Each `result.json` has `verified: true`.
The branch was merged with the later docs-only `main` (`3e565c4d1`) before the PR; `git diff e66b1dca2 HEAD` over
`src`, `lib`, `CMakeLists.txt` and `cmake` is empty, so the gate and verifications stand for the merged tree.

## Impact measurement (behaviour-active, measured on the pair as a unit)

Frozen v1 cohort, seeds 11/29/47, cap 20000, wall 120, stall 30; observer builds of before `afdd7921d` and after
`e66b1dca2`. All three traces diverge.

| seed | before | after |
| --- | --- | --- |
| 11 | alive at cap, level 3, HP 28 | alive at cap, level 3, HP 28 |
| 29 | alive at cap, level 5, HP 58 | alive at cap, level 3, HP 49 |
| 47 | **death by a cave spider, turn 11780** (issue 255) | **alive at cap**, level 3, HP 58 |

Seed 47 is the death recorded under patch 0056; it does not occur with the pair. The two patches were measured together,
so the change cannot be attributed to one of them, and three fixed seeds are not statistics: no survival, progress or
population claim is made in either direction. Archive `artifacts/rf-measurement.tar.gz` (sha256 `7e3b93430879fbf7c759f3b998c49b4bb3d7a73c3d73e445f519f72517870119`, 55373794 bytes,
95 members, build/source trees and saves excluded; ignored by Git, kept locally).

## Scope and limits

Gate once on the stacked tip, not per patch; fixtures isolate the flow result, not play. Normal phases only; the
accumulated prior-fixture matrix was not rerun (see the milestone 6 contract). The 0064/0065 cohort effect is a pair
measurement. The parked branch carried a handover note during the owner's brief pause; it was removed when the work
resumed.

## Exact next work

PR from `task/carry-revflow`, hosted checks, merge; mark 268/269 carried/open+patch-active; receipts on each, on
sources 146/345 and on checkpoint 1; note the seed-47 result on #255. Next: swapacid-199; recheck the source board.
