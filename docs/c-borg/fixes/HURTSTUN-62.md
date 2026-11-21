# Hurtstun-62 carry: verified no-behaviour-change cleanup

Consumer [329](https://github.com/diegoami/better-borg-c/issues/329),
canonical source
[62](https://github.com/diegoami/neo-angband-tracker/issues/62)
(CBUG-0046, hurtstun-62; source fix `f1aca2e9a`, evidence fixture
`2cb17f801`). **Cleanup, no behaviour change; owner decision
2026-11-08: delete the dead block.** Applied as a `cherry-pick -x` of
source `f1aca2e9a3d4e6ffe7e301c9aa750173dd3883bd`, patch
`patches/0085-hurt-blow-dead-stun-test.patch` (applied commit
`c55e11c92c`).

## What is removed

In `borg_danger_physical()` the `MONBLOW_HURT` case sets
`z = d_dice * d_side` and on the next line tests
`z > d_dice * d_side` for a +200 stun penalty. Neither value changes in
between, so the test is never true and the +200 is never added. The
case it was meant for — blows with very few sides and many dice that
tend to knock out (10d2 kicks) — is already covered by the following
`d_side < 3 && d_dice > 5` test, which adds 400 and is kept.

## Pins and provenance

Before: merged main `1bd2e05a5` (snapshotted build `build/native-before`,
gate result `artifacts/native-before/result.json`, 965/965 + 5/5 at that
commit). `borg-danger.c` is otherwise pristine on the product. The
verifier proves that the source diff applied to the before module equals
the product module and that the applied commit is the only `src`/`lib`
difference from the before commit. Fixture: kit `2cb17f801` at head
`803182465`, asserted byte-identical to the run pin.

## Verified results (`artifacts/hs62/scenarios`, 2026-11-09)

- `mystic` / `ant` / `orc` / `giant` — all four pass in both phases with
  **byte-identical logs** and identical rows (danger and full both
  1680/2/8/64): the removal changes nothing observable, which is the
  contract of a cleanup carry.
- `result.json` records `verified: true` with all commands, pins and
  hashes. The verifier passed on its first complete run.

## Scope and limits

Four blow-profile danger scenarios (mystic/ant/orc/giant) driving the
production physical-danger path. The contract is the absence of change:
every mode passes in both phases with byte-identical logs and identical
rows. This is a cleanup carry under the recorded owner decision, not a
bug fix.

## Exact next work

1. PR from `task/carry-hurtstun`, agent evidence review plus required
   hosted checks, then normal merge.
2. Frozen v1 cohort at the merged tip against the lightbeam tip cohort.
3. Receipts on issue 329, source 62 and checkpoint 1.
