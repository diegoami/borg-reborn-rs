# Missile no-target carry: verified product scenario, ready for adoption

Consumer [51](https://github.com/diegoami/better-borg-c/issues/51), canonical
source [254](https://github.com/diegoami/neo-angband-tracker/issues/254)
(missile-254). Fix applied as `09dd4435bd0f36d702246ad6936c856cf08a7e51`
(import of source PR288 `405e4fd4880e7bef27d6a6328765a904638c8257`, patch
`patches/0011-missile-no-target.patch`; import diff verified byte-identical
to the source commit, and pristine `e473720` + patch 0011 reproduces the
product module SHA256
`1750601167e32b6d7983a3134d52696a24d44d707a91a08d8413f5feee527385`).

## Defect

`borg_attack_aux_launch()`'s probe can run with the target list empty (the
Teleport Other defense resets it and the launch routine does not refill it)
and `borg_simulate` off (any acted attack/defense leaves it off; it
zero-inits off). Every candidate returns −1, the chosen index `b_k` stays
−1, and the "nothing to use" guard tested `b_n < 0`, which cannot be true
once `b_n` initializes to zero: the real call read `borg_items[-1].desc`
and queued a fire key, a quiver character computed from index −1 and the
target key. The one-guard correction (`b_k < 0`) also makes the dead guard
live. borg-fight-attack.c is otherwise pristine in the product, so the
import is exactly the reviewed guard.

## Pins

Before build: ten-carry `b72ad8eae676ba7c2b9d90026eb0e2d4a2468540` (retained
tree `../glyph-tracker`, gate at `40b002da`, source-identical to
`b72ad8eae` over src/lib). Evidence fixture: unchanged source pin
`8b30eb584` (`docs/audit/c-reports/missile-254/`, PR289). ASan runs use
the source-evidence options `detect_leaks=0:intercept_strcmp=0` (disclosed
pristine init name-sort strcmp overread); the tracked read is
compiler-instrumented.

## Verified results (`artifacts/missile/scenarios`, 2026-11-06)

Command (under `/tmp/angband-validation.lock`):

```sh
python3 tools/verify-missile-no-target.py \
  --before-build ../glyph-tracker/build/native \
  --output artifacts/missile/scenarios
```

- normal-before `no-target`: exit 0, 0/1, `return=0 keys=3` — the guard is
  dead, a fire sequence is queued from index −1.
- normal-after `no-target`: exit 0, 1/1, `return=0 keys=0`.
- `live-shot` (`return=4 keys=11`) and `simulated` (`return=4 keys=0`):
  1/1 on every phase, byte-identical logs before/after (identical SHA256
  pairs in `result.json`).
- asan-before `no-target`: exit 1, **heap-buffer-overflow READ of size 1**
  through `my_strcpy` (z-util.c:482) in `borg_attack_aux_launch`
  (`before-attack.c:2148`, the pre-fix object linked into the fully
  instrumented core); controls clean.
- asan-after: all three modes 1/1, no sanitizer report.
- `result.json` records `verified: true` with all commands, binary/object
  hashes and the fixture SHA256.

Native gate on this core: **965/965 unit + 5/5 frontend**
(`artifacts/native/result.json`, product binary
`9e10e12237c6…`). Regression matrix
(`artifacts/missile-carry/regression.py`): all **63 prior native fixture
cases whole-log identical** to their baselines — 49 older + 6 source279 +
5 source64 against the nine-carry logs, and the tenth carry's 3 glyph modes
against its verified after-logs. 63 cumulative fixture cases, not 63
independent encounters. Tooling 29/29.

## Scope and limits

Bounded native producer scenario on the real launch chooser; the
empty-target state is constructed directly (the production path that leaves
it — a Teleport Other defense reset followed by the shot-lining probe — is
documented in the source evidence, not replayed here). Not an autonomous
reachability, frequency or survival claim. Danger-free player at full HP;
no monster AI participation. The failed-before behavior (3 queued keys) is
retained in the phase logs.

## Exact next work

1. PR from `task/carry-missile-no-target`, agent evidence review plus
   required hosted checks, then normal merge (no per-PR external review
   stop).
2. On merged main: measure actual frozen v1 seeds 11/29/47, cap 20000,
   wall 120, stall 30; keep all outcomes.
3. Mark 51 carried/open+patch-active; post the Verification section;
   reply on source 254; checkpoint receipt on issue 1; evidence archive
   under `artifacts/missile-carry/`.
4. Then recheck source67 (tp-other skip list) and source248 (dispel
   rating) — the remaining demonstrated pairs, in severity order.
