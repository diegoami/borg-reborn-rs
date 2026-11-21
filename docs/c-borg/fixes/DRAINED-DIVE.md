# Drained-Borg dive guard carry: verified product scenario

Consumer [263](https://github.com/diegoami/better-borg-c/issues/263),
canonical source [235](https://github.com/diegoami/neo-angband-tracker/issues/235)
(drainedive-235). Fix applied as `eb89278d7c312d236703e39b90d865b44397ec10`
(`cherry-pick -x` of source `6d909bf5216c1e6c5ed199595d9ed3e5baf4b0d8`, patch
`patches/0061-drained-no-dive.patch`).

## Defect

`borg_leave_level()`'s "Do not dive when drained" guard was gated on `g`, which is
non-zero only when an earlier branch already decided to head up, so a drained Borg
that had decided to dive was never stopped. The sibling "full of items" guard is
unconditional and the original 2011 borg cancelled a decided dive here. The correction
applies the guard unconditionally. Behaviour-active for a Borg with drained experience.
No survival claim.

## Pins and provenance

Before: `origin/main` `828d5eec26626f0c588b83497b189fbb606e62bb` (build tree copied aside before the fix).
`borg-think-dungeon-util.c` carries earlier patches elsewhere; the verifier proves the
source diff applied to the before module equals the product module and that this file is
the only `src`/`lib`/build-file difference. Evidence fixture: source evidence head
`300d18cdd256f6055f2ae905057e0254810c9b07`; runtime files byte-identical to run pin
`557c85260f8b836ab2c8725fe8baa0910be38261` (asserted).

## Verification (`artifacts/dd235/scenarios`, 2026-11-08)

```sh
git remote add source-tracker https://github.com/diegoami/neo-angband-tracker.git
git fetch source-tracker task/drainedive-235 task/drainedive-235-evidence
flock /tmp/angband-validation.lock python3 tools/verify-native.py --jobs 10
flock /tmp/angband-validation.lock python3 tools/verify-drainedive.py \
  --before-build BEFORE_BUILD_TREE --fix eb89278d7c312d236703e39b90d865b44397ec10 \
  --output artifacts/dd235/scenarios
```

Native Human Warrior, seed 235, character level 49, depth 15, prepared through depth 20,
real experience drain; `borg_leave_level(false)` runs once.

| phase | exit | result | `dive_notes` | `drained` |
| --- | --- | --- | --- | --- |
| before | 0 | **0/1** | 1 ("power dive") | 1 |
| after | 0 | **1/1** | 0 | 1 |

Rows equal the source evidence's recorded rows value for value. Native gate on `eb89278d7`:
**965/965 unit + 5/5 frontend**; tooling 121 pass. `result.json` has `verified: true`.

## Impact measurement

Frozen v1 cohort, seeds 11/29/47, cap 20000, wall 120, stall 30; observer builds of before
`828d5eec26626f0c588b83497b189fbb606e62bb` and after `eb89278d7`. All three traces are byte-identical (11 and 29 alive at the
cap, 47 dies to a cave spider at turn 11780, the lineage of patch 0056 / issue 255). Neutral
on this cohort, whose characters are never experience-drained. Not a benefit claim. Archive
`artifacts/dd235-measurement.tar.gz` (sha256 `ebd5bb55b229b45314f90fd6914b0d79e03d3ff30cc1f34909c9b6a11f2bf1de`, 55325829 bytes, 95 members, build/source
trees and saves excluded; ignored by Git, kept locally).

## Scope and limits

One think pass at fixed state with a single mode and no separate control; the sibling
guard's unconditional form is the source's argument for correctness. Normal phases only;
prior-fixture matrix not rerun.

## Exact next work

PR from `task/carry-drainedive`, hosted checks, merge; mark 263 carried/open+patch-active;
receipts on 263, source 235, checkpoint 1. Remaining: prepreason-234 (source fixture still
pending, not carried), swapacid-199, spastic-146, glyphflow-345, respois-184/185.
