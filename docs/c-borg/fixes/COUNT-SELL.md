# Count-sell unidentified-item carry: verified product scenario

Consumer [261](https://github.com/diegoami/better-borg-c/issues/261),
canonical source [134](https://github.com/diegoami/neo-angband-tracker/issues/134)
(countsell-134 / CBUG-0118). Fix applied as
`93cbfbda08f91693a7c9bbb1ef9eb8df14860333` (`cherry-pick -x` of source
`026a42d3fcdfcbedaaef12c6f39200e38ecbd2ed`, patch
`patches/0060-count-sell-skips-unidentified-single.patch`).

## Defect

`borg_count_sell()` returned 0 at once when it met an unidentified item that is not
part of a pile, discarding everything already counted, so a full pack with one such
item looked like nothing to sell and the Borg dived instead of selling first. The
correction skips that item (`continue`). Behaviour-active in town restock and dive
decisions. No survival claim.

## Pins and provenance

Before: `origin/main` `d4f9bb5c92179fcc4107d357af3882738d482aff` (build tree copied aside before the fix).
`borg-store-sell.c` carries no earlier patch over this line; the verifier proves the
source diff applied to the before module equals the product module and that this file
is the only `src`/`lib`/build-file difference. Evidence fixture: source evidence head
`6aedfa6f16479584a9c59c6bd6dbdc8408f624a8`; runtime files byte-identical to run pin
`68b23b2a1b2eeb94d70a86e98375aaddaadf0951` (asserted).

## Verification (`artifacts/cs134/scenarios`, 2026-11-08)

```sh
git remote add source-tracker https://github.com/diegoami/neo-angband-tracker.git
git fetch source-tracker task/countsell-134 task/countsell-134-evidence
flock /tmp/angband-validation.lock python3 tools/verify-native.py --jobs 10
flock /tmp/angband-validation.lock python3 tools/verify-countsell.py \
  --before-build BEFORE_BUILD_TREE --fix 93cbfbda08f91693a7c9bbb1ef9eb8df14860333 \
  --output artifacts/cs134/scenarios
```

| mode | before | after |
| --- | --- | --- |
| `full-unid` (full pack, one unidentified single item) | 0/1, `sellable=0`, goes down | **1/1**, `sellable=15`, stays up to sell |
| `full` (no unidentified item) | 1/1, `sellable=15` | 1/1; log byte-identical |
| `light` (nearly empty pack) | 1/1, `sellable=0`, dives | 1/1, `sellable=2`, still dives |

All rows equal the source evidence's recorded rows value for value. Native gate on
`93cbfbda0`: **965/965 unit + 5/5 frontend**; tooling 121 pass. `result.json` has
`verified: true`.

## Impact measurement

Frozen v1 cohort, seeds 11/29/47, cap 20000, wall 120, stall 30; observer builds of
before `d4f9bb5c92179fcc4107d357af3882738d482aff` and after `93cbfbda0`. All three traces are byte-identical before and
after (11 and 29 alive at the cap, 47 dies to a cave spider at turn 11780, the lineage
of patch 0056 / issue 255). The correction is neutral on this cohort, whose Warriors
never meet an unidentified single item with a sellable pack. Not a benefit or survival
claim. Archive `artifacts/cs134-measurement.tar.gz` (sha256 `d37d81e9b54398ee90b8f4c8b22bf7b4254f5b7dd8455dc04074076787455c8f`, 55316814 bytes, 95
members, build/source trees and saves excluded; ignored by Git, kept locally).

## Scope and limits

The fixture isolates the sell count and the dive decision, not shopping play. Normal
phases only; prior-fixture matrix not rerun.

## Exact next work

PR from `task/carry-countsell`, hosted checks, merge; mark 261 carried/open+patch-active;
receipts on 261, source 134, checkpoint 1. Next: drainedive-235, prepreason-234, swapacid-199,
spastic-146, glyphflow-345, respois-184/185; recheck the source board.
