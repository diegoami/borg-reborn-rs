# Excessive-danger flee carry: verified product scenario, cohort divergence recorded

Consumer [247](https://github.com/diegoami/better-borg-c/issues/247),
canonical source [156](https://github.com/diegoami/neo-angband-tracker/issues/156)
(dangerflee-156 / CBUG-0140). Fix applied as
`26811540dd23b382f0a80ab2f3011806f1f81b50` (`cherry-pick -x` of source
`007d45483832b2792ddde1b2c368b67975365496`, patch
`patches/0053-flee-excessive-danger.patch`). Owner decision on the source issue:
option 1, `ready_morgoth != 1`, matching the comment.

## Defect

`borg_caution()`'s "Fleeing (excessive danger)" branch is documented as "do not
flee level if going after Morgoth or fighting a unique" but tested
`ready_morgoth == 1`, so it fired only for a Borg that WAS going after Morgoth
and never for an ordinary one. The correction inverts that test. Behaviour-active:
an ordinary Borg whose danger exceeds twice its hit points now flees the level
(desperate stairs) where it did not; a Morgoth-ready Borg no longer takes the
branch. No survival claim.

## Pins and provenance

Before: `origin/main` `120ca9916d0b0813171112f9345375fd7745d76b` (build tree
copied aside before the fix). `borg-caution.c` carries no earlier patch over this
line; the verifier proves before module + source diff == product module and that
this file is the only `src`/`lib`/build-file difference. Evidence fixture: source
evidence head `7d796742e9ad496695baa24f9b076f203a5d74c6`, unchanged; runtime files
byte-identical to run pin `809e556b39e47c2dd13c6dd53aa42b7ac8329010` (asserted).

## Verification (`artifacts/df156/scenarios`, 2026-11-08)

```sh
git remote add source-tracker https://github.com/diegoami/neo-angband-tracker.git
git fetch source-tracker task/dangerflee-156 task/dangerflee-156-evidence
flock /tmp/angband-validation.lock python3 tools/verify-native.py --jobs 10
flock /tmp/angband-validation.lock python3 tools/verify-dangerflee.py \
  --before-build BEFORE_BUILD_TREE --fix 26811540dd23b382f0a80ab2f3011806f1f81b50 \
  --output artifacts/df156/scenarios
```

| mode | before | after |
| --- | --- | --- |
| `excessive` (danger 20 > 2 x hp 8, no unique, not Morgoth-ready) | 0/1, no flee, no stairs | **1/1**, flee + desperate stairs |
| `morgoth` (Morgoth-ready) | 0/1, flee + stairs (inverted condition) | **1/1**, no flee, no stairs |
| `hurt` (hp 15, danger within bounds) | 1/1 | 1/1; log byte-identical |

All rows equal the source evidence's recorded rows value for value. Native gate on
`26811540d`: **965/965 unit + 5/5 frontend**; tooling 121 pass. `result.json` has
`verified: true`.

## Impact measurement: behaviour-active, divergence and one death recorded

Frozen v1 cohort, seeds 11/29/47, cap 20000, wall 120, stall 30; observer builds of
before `120ca9916` and after `26811540d`.

| seed | before | after | traces |
| --- | --- | --- | --- |
| 11 | turn_cap, alive | turn_cap, alive (2 flee notes) | diverge (5476 vs 5417 lines) |
| 29 | turn_cap, alive | **death by a cutpurse at game turn 15149** (2 flee notes) | diverge (4076 vs 4643 lines) |
| 47 | turn_cap, alive | turn_cap, alive (0 flee notes) | identical |

The "Fleeing (excessive danger)" note appears 0 times before and twice in the
diverging runs. Seed 29 diverges at turn 12120 (level-2 Borg, floating eye, danger
200 against 33 hp: the corrected branch flees the level, as intended) and later dies
at depth 2, character level 4, at HP 1/53 attacking an adjacent cutpurse while "Fleeing and leaving the
level (scaryguy)". The death is downstream of a different trajectory, not of the
corrected branch directly, and is one frozen seed: it neither shows the correction
harmful nor refutes it, and it is not a population estimate. It is recorded as
observed and filed for diagnosis (see the exact next work). Archive
`artifacts/df156-measurement.tar.gz` (sha256
`2b5dd254090289e61fd6d1d077d3570a55405f4343ee7d5016876c674ba6a1af`, 55283747 bytes,
95 members, build/source trees and saves excluded; ignored by Git, kept locally).

## Scope and limits

* The fixture isolates one decision at fixed hit points and danger; it does not
  show downstream play. Normal phases only; prior-fixture matrix not rerun.
* Three seeds are a divergence watch, not statistics.

## Exact next work

1. PR from `task/carry-dangerflee`, hosted checks, normal merge (owner rule: merge
   after agent evidence review; this is the owner-decided behaviour of source156).
2. Investigation issue for the seed-29 cohort death (frozen, replayable from
   `input.txt`); no strategy change without evidence. If the owner prefers, revert
   by removing patch 0053; nothing else depends on it.
3. Mark 247 carried/open+patch-active; receipts on 247, source 156 and checkpoint 1.
