# Clock-overflow graceful reset carry: verified product scenario

Consumer [245](https://github.com/diegoami/better-borg-c/issues/245),
canonical source [246](https://github.com/diegoami/neo-angband-tracker/issues/246)
(clockreset-246). Fix applied as `49f9c19df2b7157594498eb81091c8f4c6dbdbf6`
(`cherry-pick -x` of source `7cc0ebbd2016afeaaf57c5e976251b67f7c0e147`, source PR 415,
patch `patches/0052-clock-overflow-graceful-reset.patch`). Owner decision on the
source issue: option 1, reset first, abort as failsafe.

## Defect

`borg_think_dungeon()` aborted the Borg (`borg_oops("clock overflow")`) at
`BORG_TIME_MAX - 3000`, about 2900 turns before the graceful reset in
`borg_think()` could run, so that reset was unreachable. The correction
performs the same `borg.time.now = 1` assignment at the threshold and keeps
playing; `borg_think()`'s overflow check stays as a failsafe. Behaviour-active
only when the Borg clock approaches its limit (about 2.1 billion ticks). No
survival claim.

## Pins and provenance

Before: `origin/main` `068d34022a77949ec5157fdb62bb3b1e3e7242cb` (build tree copied aside before the fix was
applied). `borg-think-dungeon.c` carries no earlier patch over these lines;
the verifier proves that the source diff applied to the before module equals
the product module and that this file is the only `src`/`lib`/build-file
difference from the before commit. Evidence fixture: source evidence head
`6d4859904037aa987577a8bcab522d5ebd4ae17a`, unchanged; `scenario.c`, `borg.txt`
and `profile` are byte-identical to the run pin
`85ff225404d99acfdf22568b4bc5ec64afd4218b` (asserted).

## Verification (`artifacts/cr246/scenarios`, 2026-11-08)

```sh
git remote add source-tracker https://github.com/diegoami/neo-angband-tracker.git
git fetch source-tracker task/clockreset-246 task/clockreset-246-evidence
flock /tmp/angband-validation.lock python3 tools/verify-native.py --jobs 10
flock /tmp/angband-validation.lock python3 tools/verify-clockreset.py \
  --before-build BEFORE_BUILD_TREE --fix 49f9c19df2b7157594498eb81091c8f4c6dbdbf6 \
  --output artifacts/cr246/scenarios
```

Native Human Warrior, seed 246, depth 9, glowing arena, no monsters; the Borg
clock is set to the abort threshold `BORG_TIME_MAX - 3000` and one production
`borg_think_dungeon()` pass runs.

| phase | exit | result | clock after | Borg active |
| --- | --- | --- | --- | --- |
| before | 0 | **0/1** (`borg.time.now < 1000` fails) | 2147480647 | 0 (aborted) |
| after | 0 | **1/1** | 1 | 1 |

Both rows equal the source evidence's recorded rows value for value.
`result.json` records `verified: true` with commands and hashes. Native gate
on `49f9c19df`: **965/965 unit + 5/5 frontend**, read numerically from
`artifacts/native/result.json`. Tooling tests 121 pass.

## Scope and limits

* One think pass at the threshold; it does not show the following turns of
  autonomous play.
* No frozen v1 cohort run: the cohort cap is 20000 turns, far below the
  threshold, so the corrected branch cannot execute there and a run would
  prove nothing about it.
* Normal phases only (no memory error); prior-fixture matrix not rerun, the
  full native gate holds.

## Exact next work

PR from `task/carry-clockreset`, agent evidence review and hosted checks,
normal merge; mark 245 carried/open+patch-active; receipts on 245, source 246
and checkpoint 1. Next: dangerflee-156, recallmana-157, deskstair-153,
drainedive-235, prepreason-234, swapacid-199, spastic-146, glyphflow-345.
