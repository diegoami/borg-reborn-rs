# Munchkin stair-range gate carry: verified product scenario

Consumer [242](https://github.com/diegoami/better-borg-c/issues/242),
canonical source [245](https://github.com/diegoami/neo-angband-tracker/issues/245)
(stairrange-245). Fix applied as `30bc61d37a72933a30f0c5425ce3993208a2e24f`
(`cherry-pick -x` of source `c16b3e9d14470dd19ef884f37f9eca2f7cc7d37f`, source PR 413,
patch `patches/0051-munchkin-stair-range-gate.patch`). Owner decision on the
source issue: option 1, apply the adjustment only when it increases the range.

## Defect

`borg_think_dungeon_munchkin()` adds `(clevel - 10) + (10 - depth)` to the base
stair range of 8 whenever the depth is at most 10. Below character level 10
the first term is negative, so the "increase" shrinks the range: a level-5
Borg at depth 10 gets range 3 (a level-1 Borg, -1). With the shrunk range no
visible staircase counts as close and the munchkin Borg stops power-climbing
toward stairs it can see. The correction adds `&& borg.trait[BI_CLEVEL] > 10`;
at level 10 and above the previous growth is unchanged. Behaviour-active in
munchkin mode below level 10. No survival or frequency claim.

## Pins and provenance

Before: `origin/main` `eea2eee96412280d6406f772ac2f73d8975f7ba4` (build tree
copied aside before the fix was applied). Product: `30bc61d37`, merged as `ae64c39cb8cf90ebf5f55b7627414847c155b4d2` (PR 243). `borg-think-dungeon.c`
carries no earlier patch over these lines; the verifier still proves that the
source diff applied to the before module equals the product module, and that
the only `src`/`lib`/build-file difference from the before commit is that file.
Evidence fixture: source evidence head `5da8217435bb693c3aabbff9d75520f96f355c61`,
unchanged; `scenario.c`, `borg.txt` and `profile` are byte-identical to the run
pin `3c9a75e2023386863db8a21be16db5b8c12a840a` (asserted).

## Verification (`artifacts/sr245/scenarios`, 2026-11-08)

```sh
git remote add source-tracker https://github.com/diegoami/neo-angband-tracker.git
git fetch source-tracker task/stairrange-245 task/stairrange-245-evidence
flock /tmp/angband-validation.lock python3 tools/verify-native.py --jobs 10
flock /tmp/angband-validation.lock python3 tools/verify-stairrange.py \
  --before-build BEFORE_BUILD_TREE --fix 30bc61d37a72933a30f0c5425ce3993208a2e24f \
  --output artifacts/sr245/scenarios
```

Native Human Warrior, seed 245, level 5, depth 10, no monsters, a known up
staircase five grids north (outside the shrunk range 3, inside the base range
8); the production munchkin decision runs once.

| phase | exit | result | `goal_less` | `climb_decision` |
| --- | --- | --- | --- | --- |
| before | 0 | **0/1** (`require(borg.goal.less)` fails) | 0 | 0 |
| after | 0 | **1/1** | 1 | 1 |

Both rows' `STATE` values (`clevel=5 depth=10 fleeing=1 munchkin_mode=1 rc=1`)
equal the source evidence's recorded before/after rows value for value.
`result.json` records `verified: true` with commands and hashes.

Native gate on the product commit `30bc61d37`: **965/965 unit + 5/5 frontend**
read numerically from `artifacts/native/result.json`. Tooling tests: 121 pass.

## Scope and limits

* One configuration (level 5, depth 10, stair at distance 5). The fixture has
  no level >= 10 control, so "unchanged at level 10 and above" rests on the
  one-token gate and the source's statement, not on a native control here.
* Normal phases only; no sanitizer phase (range arithmetic, no memory error).
* The regression matrix over prior fixtures was not rerun; the patched module
  has no earlier carry over these lines, and the full native gate holds.
* Behaviour-active in munchkin mode below level 10: the frozen v1 cohort is
  watched for divergence after merge rather than assumed neutral. Not a
  survival or progress claim.

## Impact measurement (behaviour-active in munchkin mode below level 10)

Frozen v1 cohort, seeds 11/29/47, cap 20000, wall 120, stall 30, observer
builds of before `eea2eee96` and merged `ae64c39cb` prepared with
`tools/measure-native.py prepare --observe`. All six runs end `turn_cap`
"(alive and well)". Per seed, `trace.txt`, `input.txt`, `terminal.log` and the
user files are byte-identical between before and after (trace sha256 prefixes
`030f9aba` / `cb80b119` / `8dcaffb7`); `result.json` differs only in paths,
timestamps and binary hashes. The fix is therefore neutral on this cohort:
the v1 Warriors never reach a sub-level-10 munchkin stair decision within
20000 turns. That is a narrow regression result, not evidence of benefit and
not a survival claim. Archive `artifacts/sr245-measurement.tar.gz`
(sha256 `c5da43b0582f55d42d9bbf5a572361a2b49f9b84a12e2af9c6e6b2df8d6316ba`,
55272513 bytes, 95 members, build/source trees and saves excluded; ignored by
Git, retained locally).

## Exact next work

1. PR from `task/carry-stairrange`, agent evidence review and required hosted
   checks, then normal merge.
2. Done: PR 243 merged `ae64c39cb`; v1 cohort measured (identical traces).
3. Done: 242 marked carried/open+patch-active; receipts on 242, source 245
   and checkpoint 1.
4. Next carries: clockreset-246, drainedive-235, prepreason-234, swapacid-199,
   spastic-146, glyphflow-345 (apply cleanly to main); staircycle-403 awaits
   the consumer 204 correction boundary; tapunlife-249 stays masked (consumer 140).
