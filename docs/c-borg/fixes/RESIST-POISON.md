# Resist Poison reward counter carries (sources 184, 185): verified product scenarios

Two one-line corrections in `borg_power_inventory()` (`src/borg/borg-power.c`), adopted as
consecutive patches in one PR with a before-build snapshot at each step. They fix two different
stale-counter spots and both are needed.

| patch | consumer | source | applied | source commit |
| --- | --- | --- | --- | --- |
| 0062 | [266](https://github.com/diegoami/better-borg-c/issues/266) | [184](https://github.com/diegoami/neo-angband-tracker/issues/184) respois-184 | `089acfae4735f1b76d2fa2723cdbc44786647c4f` | `5e8d4c5e6c1006ce4c90e21a973b0f95bb9fa0db` |
| 0063 | [267](https://github.com/diegoami/better-borg-c/issues/267) | [185](https://github.com/diegoami/neo-angband-tracker/issues/185) respoiswar-185 | `b81495995bad7e1c17cbf7ce7b51d7f22a9808ea` | `c056daabf55e6a1713e68381ef4a72ec6fd349b9` |

## Defects

* **184:** the Resist Poison potion reward loop (`for (; k < 4 ...)`) reuses a counter `k` left by the
  food loop, so with food in the low range the reward is lost (900 becomes 0). `k = 0` is set first.
* **185:** the warrior's Resist Poison reward continues the Resist Cold count, so cold-resistant warriors
  are under-rewarded (2400 becomes 900). `k = 0` is set before the poison loop.

Inventory-power values steer shopping and what the Borg keeps; behaviour-active only through those
rewards. No survival claim.

## Pins and provenance

Before: `origin/main` `31e71f0873c3166b195499bf86b5457ec962b2b5`. Evidence fixtures unchanged (heads `c94b12b3c58caeb3310a772f2322b76baa711ebe`,
`58a366e8b9bdc886c9c94292271ea0313e678906`; runtime files byte-identical to the run pins
`af01d73cc`, `6f130a4ee`, asserted). 0062 provenance: source diff
applied to the before module equals the product module; 0063 stacks in the same function, so the source
change merged with `git merge-file` onto the before module equals the product module. Each verifier asserts
that `borg-power.c` is the only `src`/`lib`/build-file difference from its before commit.

## Verification (`artifacts/respois`, `artifacts/respoiswar`; 2026-11-08)

```sh
git remote add source-tracker https://github.com/diegoami/neo-angband-tracker.git
git fetch source-tracker task/respois-184 task/respois-184-evidence task/respoiswar-185 task/respoiswar-185-evidence
# build origin/main, snapshot build/native (SNAP0); build the 0062 commit, snapshot (SNAP1); build the tip, then
flock /tmp/angband-validation.lock python3 tools/verify-respois.py --before-build SNAP0 --fix 089acfae4735f1b76d2fa2723cdbc44786647c4f --output artifacts/respois/scenarios
flock /tmp/angband-validation.lock python3 tools/verify-respoiswar.py --before-build SNAP1 --fix b81495995bad7e1c17cbf7ce7b51d7f22a9808ea --output artifacts/respoiswar/scenarios
flock /tmp/angband-validation.lock python3 tools/verify-native.py --jobs 10   # on the tip
```

| source | mode | before | after |
| --- | --- | --- | --- |
| 184 | `apples` (low food) | 0/1, `potion_reward=0` | **1/1**, `potion_reward=900` |
| 184 | `rations` | 1/1, 900 | same; log byte-identical |
| 185 | `cold` (4 Resist Cold potions) | 0/1, `potion_reward=900` | **1/1**, `potion_reward=2400` |
| 185 | `plain` | 1/1, 2400 | same; log byte-identical |

All rows equal the source evidence's recorded rows value for value (the `potion_reward` figure is
printed on its own STATE line; the verifier merges all matching lines). Native gate on the tip
`b81495995`: **965/965 unit + 5/5 frontend**; tooling 121 pass. Each `result.json` has `verified: true`.

## Retained verifier note

The first stepwise run reused a `build/native` left at the fixed tip as its "before" tree, so the 0062
"before" row passed; the assertion rejected it and nothing was weakened. The before-tree was rebuilt from
`origin/main` and the steps rerun.

## Scope and limits

Gate once on the stacked tip; fixtures isolate the power value, not the shopping or keeping decisions that
read it. Normal phases only; prior-fixture matrix not rerun. No cohort run (the v1 Warriors rarely hold
poison-resistance potions); the power change can shift shopping, so a cohort divergence is possible and not
excluded by this document.

## Exact next work

PR from `task/carry-respois`, hosted checks, merge; mark 266/267 carried/open+patch-active; receipts on
each, on sources 184/185 and on checkpoint 1. Next: spastic-146 and glyphflow-345 (reverse flow), swapacid-199.
