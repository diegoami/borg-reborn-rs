# Back-away per-grid judgement carry: verified product scenario, mixed cohort result

Consumer [254](https://github.com/diegoami/better-borg-c/issues/254),
canonical source [155](https://github.com/diegoami/neo-angband-tracker/issues/155)
(backaway-155 / CBUG-0139). Fix applied as
`86d01e947126e81b2debae4a3d7fe14517e29d9f` (`cherry-pick -x` of source
`df5361ad5cc3977c75a86061ad0d944568885d77`, patch
`patches/0056-back-away-per-grid.patch`).

## Defect

The back-away scan in `borg_caution()` leaked its "adjacent to a monster" state
from one candidate grid to the next (the flag was never reset per grid) and a trap
ended the whole scan (`break` instead of `continue`). The correction resets the flag
per grid and skips only the trapped grid. Behaviour-active when a hurt Borg
considers backing away. No survival claim.

## Pins and provenance

Before: `origin/main` `f0e10ee1236a800f9b538e609b24afca4b864ff5` (build tree copied aside before the fix).
`borg-caution.c` also carries patches 0053-0055 elsewhere; the verifier proves before
module + source diff == product module and that this file is the only
`src`/`lib`/build-file difference. Evidence fixture: source evidence head
`c7bd31508cf6a8f001b408b810263fb297d0ddd2`; runtime files byte-identical to run pin
`fa0293e587d8ab7e8e2e90e5fade853a2cfcd0ef` (asserted).

## Verification (`artifacts/ba155/scenarios`, 2026-11-08)

```sh
git remote add source-tracker https://github.com/diegoami/neo-angband-tracker.git
git fetch source-tracker task/backaway-155 task/backaway-155-evidence
flock /tmp/angband-validation.lock python3 tools/verify-native.py --jobs 10
flock /tmp/angband-validation.lock python3 tools/verify-backaway.py \
  --before-build BEFORE_BUILD_TREE --fix 86d01e947126e81b2debae4a3d7fe14517e29d9f \
  --output artifacts/ba155/scenarios
```

| mode | before | after |
| --- | --- | --- |
| `monster` (leaked monster state) | 0/1, `backed_away=0` | **1/1**, `backed_away=1` |
| `trap` (trap ends the scan) | 0/1, `backed_away=0` | **1/1**, `backed_away=1` |
| `healthy` (control) | 1/1, `backed_away=0` | 1/1; log byte-identical |

All rows equal the source evidence's recorded rows value for value. Native gate on
`86d01e947`: **965/965 unit + 5/5 frontend**; tooling 121 pass. `result.json` has
`verified: true`.

## Impact measurement (behaviour-active): mixed, recorded as observed

Frozen v1 cohort, seeds 11/29/47, cap 20000, wall 120, stall 30; observer builds of
before `f0e10ee1236a800f9b538e609b24afca4b864ff5` and after `86d01e947`.

| seed | before | after | traces |
| --- | --- | --- | --- |
| 11 | alive at cap | alive at cap | identical |
| 29 | death by a cutpurse, turn 15149 (patch 0053 lineage, issue 249) | **alive at cap**, level 5, HP 58 | diverge |
| 47 | alive at cap, level 3, HP 58 | **death by a cave spider, turn 11780**, depth 2, level 3 | diverge |

Seed 47 first diverges at turn 5549 where a level-2 Borg logs "Backing up to 59,41
(12 > 6)". The patch saves one frozen seed and loses another: two of three alive
before and after. Three seeds are not statistics; no benefit or harm claim is made
either way. The seed-47 death is filed for diagnosis as issue 255. Archive
`artifacts/ba155-measurement.tar.gz` (sha256 `1e269875289691c6a0d6a37b04fa58a843cd76ad6051e21d0ee5ba1abb7d8c6a`, 55300803 bytes, 95 members,
build/source trees and saves excluded; ignored by Git, kept locally).

## Scope and limits

Fixtures isolate the scan decision at fixed hit points; they do not show whether the
chosen back-away grid was safer. Normal phases only; prior-fixture matrix not rerun.

## Exact next work

PR from `task/carry-backaway`, hosted checks, merge; mark 254 carried/open+patch-active;
receipts on 254, source 155, checkpoint 1. Next: cfghang-141, cfgmissing-144, cfgminus-142,
countsell-134 (new), drainedive-235, prepreason-234, swapacid-199, spastic-146,
glyphflow-345, respois-184/185.
