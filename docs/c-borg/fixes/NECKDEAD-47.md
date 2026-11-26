# Neckdead-47 carry: verified no-behaviour-change cleanup

Consumer [312](https://github.com/diegoami/better-borg-c/issues/312),
canonical source
[47](https://github.com/diegoami/neo-angband-tracker/issues/47)
(CBUG-0031, neckdead-47; source fix `e9080f35f`, evidence fixture
`530689cc7`). **Cleanup, no behaviour change; owner decision
2026-11-08, option C.** Applied as the second step of the mod-neck PR
(`cherry-pick -x` of source
`e9080f35f1d2e8c8166116d0025d2e61e499899e`, patch
`patches/0083-home-amulet-dead-tier.patch`), stacked on modcredit-70
(patch 0082).

## What is removed

`borg_power_home_aux1()` subtracts 1500 points for each amulet past the
third, then, in the else of that test, repeats the same condition
(`num_neck > 3`) for a gentler 700-point tier. The else is only reached
when `num_neck > 3` is false, so that tier could never run. The
correction removes the unreachable branch; the 1500-point penalty past
three amulets is unchanged.

## Pins and provenance

Before: the modcredit-70 step tip (snapshotted build
`build/native-before-neck`, gate result
`artifacts/native-before-neck/result.json`, 965/965 + 5/5 at that
commit). `borg-home-power.c` is otherwise pristine on the product. The
verifier proves that the source diff applied to the before module equals
the product module and that the applied commit is the only `src`/`lib`
difference from the before commit. Fixture: kit `530689cc7` at head
`3885bfcd0`, asserted byte-identical to the run pin.

## Verified results (`artifacts/nd47/scenarios`, 2026-11-08)

- `neck2` / `neck3` / `neck4` / `neck5` — all four pass in both phases
  with **byte-identical logs** (`29bd3825…`, `5f5c3f38…`, `daed454d…`,
  `9e468410…`) and identical rows (home_power 2040/3120/2625/2130;
  stores_amulet 1/0/0/0): the removal changes nothing observable, which
  is the contract of a cleanup carry.
- `result.json` records `verified: true` with all commands, pins and
  hashes. The verifier passed on its first complete run.

## Scope and limits

Four amulet-count home-score scenarios (2/3/4/5 amulets). The contract
is the absence of change: every mode passes in both phases with
byte-identical logs and identical rows. This is a cleanup carry under
the recorded owner decision, not a bug fix.

## Exact next work

1. Agent evidence review plus required hosted checks on the stacked PR,
   then normal merge.
2. Frozen v1 cohort at the merged tip against the lightwest tip cohort.
3. Receipts on issue 312, source 47 and checkpoint 1.
