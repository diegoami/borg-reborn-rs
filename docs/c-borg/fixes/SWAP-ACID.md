# Swap weapon acid resistance carry: verified product scenario

Consumer [270](https://github.com/diegoami/better-borg-c/issues/270), canonical source
[199](https://github.com/diegoami/neo-angband-tracker/issues/199) (swapacid-199). Fix applied as `91f075bdcc329ec754e10e24da3cce36c5279784`
(`cherry-pick -x` of source `eddbf5b82293ea70a76fae960bc9f8debf1579c6`, patch `patches/0066-swap-weapon-acid-element.patch`).

## Defect

The swap weapon's acid resistance was read from its electricity element (`ELEM_ELEC`), so a weapon that resists
electricity was counted as acid-resistant for preparedness and an acid-resistant one was not. The correction reads
`ELEM_ACID`. Behaviour-active through the level-30 preparedness gate for a Borg with a swap weapon. No survival claim.

## Pins and provenance

Before: `origin/main` `a4084032485a7c7f29153d0b37ce63dede3e4d5a`. Evidence fixtures unchanged (runtime files byte-identical to the run pin,
asserted). The two patches touch different files, so the plain source diff applies to each before module and the
product module equals before + source diff; each verifier asserts that its file is the only `src`/`lib`/build-file
difference from its before commit. Two stacked steps in one PR with a before-build snapshot at each.

## Impact measurement (behaviour-active, measured on the pair as a unit)

Frozen v1 cohort, seeds 11/29/47, cap 20000, wall 120, stall 30; observer builds of before `a40840324` and after
`bd9a36def`. All three traces are byte-identical and all three seeds are alive at the cap (the lineage
including the reverse-flow pair, where seed 47 is alive). Neutral on this cohort, whose Warriors do not use munchkin
mode and carry no swap weapon with an acid-relevant resistance. Not a benefit claim. Archive
`artifacts/lf-measurement.tar.gz` (sha256 `e6d452e7ed3ecb020566d7788ad250cdf91d4de63ceca46f50a80fe1f395ac13`, 55557328 bytes, 95 members, build/source trees and saves excluded;
ignored by Git, kept locally).

## Verification (`artifacts/swapacid/scenarios`, 2026-11-08)

```sh
git remote add source-tracker https://github.com/diegoami/neo-angband-tracker.git
git fetch source-tracker task/swapacid-199 task/swapacid-199-evidence
flock /tmp/angband-validation.lock python3 tools/verify-swapacid.py --before-build SNAP0 --fix 91f075bdcc329ec754e10e24da3cce36c5279784 --output artifacts/swapacid/scenarios
flock /tmp/angband-validation.lock python3 tools/verify-native.py --jobs 10   # on the tip
```

| mode | before | after |
| --- | --- | --- |
| `elec` (weapon resists electricity) | 0/1, `swap_acid=1`, `prepared30=1`, `acid_counted=1` | **1/1**, `swap_acid=0`, `prepared30=0`, `acid_counted=0` |
| `acid` (weapon resists acid) | 0/1, `swap_acid=0`, `prepared30=0` | **1/1**, `swap_acid=1`, `prepared30=1`, `acid_counted=1` |
| `fire`, `none` (controls) | 1/1 | same; logs byte-identical |

All rows equal the source evidence's recorded rows value for value. Native gate on the tip `bd9a36def`:
**965/965 unit + 5/5 frontend**; tooling 121 pass. `result.json` has `verified: true`.

## Scope and limits

Gate once on the stacked tip; the fixture isolates the resistance read and the preparedness value. Normal phases only;
the accumulated prior-fixture matrix was not rerun (see the milestone 6 contract).
