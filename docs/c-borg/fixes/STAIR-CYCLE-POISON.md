# Munchkin leaves munchkin mode while poisoned or hallucinating: verified decision-level scenario

Consumer [280](https://github.com/diegoami/better-borg-c/issues/280) (investigation record: [204](https://github.com/diegoami/better-borg-c/issues/204),
`ZOO-STALL-1045.md`), canonical source [403](https://github.com/diegoami/neo-angband-tracker/issues/403) (staircycle-403;
source PR 409, evidence PR 410). Fix applied as `bd9a36def55cde1c478136b31bf3c7c3a717d520` (`cherry-pick -x` of source `d92e06c3f964e4bba19642288ded1b331a327109`, patch
`patches/0067-munchkin-leaves-while-poisoned-or-hallucinating.patch`).

## Defect

A poisoned or hallucinating munchkin Borg could stair-cycle without advancing the world clock: stair travel does not
age the clock (level entry grants minimum move energy and the world tick is skipped during level generation), so the
timed ailment never wears off and the run freezes (the seed-1045 stall in `ZOO-STALL-1045.md`). The correction adds one
bail in the munchkin branch's early-bail sequence: while poisoned or hallucinating, leave munchkin mode and hand the turn
to the ordinary recovery path. Behaviour-active in munchkin mode while ailing. No survival claim.

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

Source pin note: the source errata supersedes fixtures `0bd2123be` with `43dee40ed05284848f3df25aa5abf19a7f776c7f`; the unchanged fixture at the evidence
head `eb36b06b524992d112cef4fc9ec51f8ea701c015` is asserted byte-identical to that pin.

## Verification (`artifacts/staircycle/scenarios`, 2026-11-08)

```sh
git fetch source-tracker task/staircycle-403 task/staircycle-403-evidence
flock /tmp/angband-validation.lock python3 tools/verify-staircycle.py --before-build SNAP1 --fix bd9a36def55cde1c478136b31bf3c7c3a717d520 --output artifacts/staircycle/scenarios
```

| mode | before | after |
| --- | --- | --- |
| `main` (poisoned and hallucinating through the real timed-effect API) | 1/2 (ailing test fails), `stair_directed=1` | **2/2**, `stair_directed=0`, Borg out of munchkin mode, turn handed to recovery |
| `clean` (healthy control) | 1/1, `munchkin_handled=1` | same; log byte-identical |

The `main` mode runs the ailing test and the clean test in one process, so its pass line is `2/2`; before, only the ailing
test fails. The verifier mirrors the source runner (`ailing_failed_as_expected`). My first generated verifier looked for
`1/1` and failed on the after row; that harness error was corrected to the source runner's own rule with no assertion
loosened (the before row must still fail). The failed attempt's result directory was overwritten by the rerun and its output
was not kept as a file; the cause is fully described here. The recorded
STATE row keeps the clean test's `poisoned=0 hallucinating=0` because the parser merges both tests' STATE lines; the raw
log shows `poisoned=1 hallucinating=1` for the ailing test, as in the source logs.
All rows equal the source evidence's recorded rows value for value. Native gate on the tip `bd9a36def`: **965/965 unit +
5/5 frontend**; tooling 121 pass. `result.json` has `verified: true`.

## Scope and limits

* **Decision level only.** The fixture isolates the munchkin decision stage, as the source does. The end-to-end stall
  (the seed-1045 loaded state at engine turn 59893, and the zoo's 152 of 200 Blackguard munchkin-d6 stalls) was **not**
  rerun on the corrected build. The Blackguard cases are described on the source issue as still to be investigated, so this
  carry is not claimed to resolve them; consumer 204 stays open as the investigation record.
* Gate once on the stacked tip; normal phases only; the accumulated prior-fixture matrix was not rerun.
