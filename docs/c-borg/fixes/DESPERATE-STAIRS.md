# Desperate nearest-staircase carry: verified product scenario

Consumer [252](https://github.com/diegoami/better-borg-c/issues/252),
canonical source [153](https://github.com/diegoami/neo-angband-tracker/issues/153)
(deskstair-153 / CBUG-0137). Fix applied as
`4a88e67f96e55ab354e94d65d8dd5b98d5171252` (`cherry-pick -x` of source
`62768100602b693ed5845f0e5749830563d7e3b3`, patch
`patches/0055-desperate-nearest-stairs.patch`).

## Defect

The desperation stair search in `borg_caution()` kept the FARTHEST free staircase
for both up and down stairs (`if (b_j >= j) continue` skips the closer ones),
contradicting its intent of heading for the nearest. The correction keeps the
nearest (and fixes the "up/down" comment). Behaviour-active whenever a desperate
Borg has two or more free staircases of a kind. No survival claim.

## Pins and provenance

Before: `origin/main` `b2c9db4798768703134bffe78014ca94a3432261` (build tree copied aside before the fix).
`borg-caution.c` also carries patches 0053/0054 elsewhere; the verifier proves
before module + source diff == product module and that this file is the only
`src`/`lib`/build-file difference. Evidence fixture: source evidence head
`993a75f8a3eef3dcc073e7dbb6b69359dd8ab635`; runtime files byte-identical to run pin
`366fd1b08c4d0c3815bbb90be2457c372e757ff4` (asserted).

## Verification (`artifacts/ds153/scenarios`, 2026-11-08)

```sh
git remote add source-tracker https://github.com/diegoami/neo-angband-tracker.git
git fetch source-tracker task/deskstair-153 task/deskstair-153-evidence
flock /tmp/angband-validation.lock python3 tools/verify-native.py --jobs 10
flock /tmp/angband-validation.lock python3 tools/verify-deskstair.py \
  --before-build BEFORE_BUILD_TREE --fix 4a88e67f96e55ab354e94d65d8dd5b98d5171252 \
  --output artifacts/ds153/scenarios
```

| mode | before | after |
| --- | --- | --- |
| `twostairs` (two free up staircases) | 0/1, `desperate_stairs=0` | **1/1**, `desperate_stairs=1` |
| `twodown` (two free down staircases) | 0/1, `desperate_stairs=0` | **1/1**, `desperate_stairs=1` |
| `onestair` (control) | 1/1, `desperate_stairs=1` | 1/1; log byte-identical |

All rows equal the source evidence's recorded rows value for value (including the
`fleeing` state flips). Native gate on `4a88e67f9`: **965/965 unit + 5/5 frontend**;
tooling 121 pass. `result.json` has `verified: true`.

## Impact measurement (behaviour-active)

Frozen v1 cohort, seeds 11/29/47, cap 20000, wall 120, stall 30; observer builds of
before `b2c9db4798768703134bffe78014ca94a3432261` and after `4a88e67f9`. Seed 11 identical. Seed 29 and seed 47
traces diverge (the "Desperate" note count changes, 8 to 12 on seed 29 and 92 to 96
on seed 47) with unchanged outcomes: seed 29 dies to a cutpurse at the same game
turn 15149 as before this patch (that death comes from patch 0053, issue 249), and
seed 47 reaches the cap alive at the same turn, depth, level and hit points. The
patch is neutral in outcome on this cohort; this is not a survival or benefit claim.
Archive `artifacts/ds153-measurement.tar.gz` (sha256
`8676c34bc35bf49660212582b2215dc83b0529553582e00f7208df93d2c42762`, 55301459 bytes,
95 members, build/source trees and saves excluded; ignored by Git, kept locally).

## Scope and limits

Fixtures isolate the staircase choice at fixed hit points; they do not show whether the
nearer staircase was reached or safer. Normal phases only; prior-fixture matrix not rerun.

## Exact next work

PR from `task/carry-deskstair`, hosted checks, merge; mark 252 carried/open+patch-active;
receipts on 252, source 153, checkpoint 1. Next: backaway-155, cfghang-141, cfgmissing-144
(new), drainedive-235, prepreason-234, swapacid-199, spastic-146, glyphflow-345,
respois-184/185.
