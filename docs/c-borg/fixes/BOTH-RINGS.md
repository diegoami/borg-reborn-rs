# Use a charged attack ring from either hand — consumer20

A Warrior has a charged Ring of Flames on the left hand and a70HP ogre in
range. The old ring-attack path requests no activation. The correction executes
the native fireball and kills the ogre, removing its70HP. The same ready ring
already works on the right. Both finding and activation must include the left
slot; either one-loop correction alone still misses that attack.

Canonical source78/CBUG-0062, discovery borg-rs; reviewed source269
46c14b2828ed751ff8be7ac67df93b28cdc15f87 imported with cherry-pick -x asc3f631068.
Final borg-item-use.c SHA25649cdc1ec633314afd3c531a86374722fa03c289a898ab22e6b36e2263e389be6
matches the proposal. Two loop bounds change; no other production logic/rules.
Original patch, exact pins and independent removal unit are in patches/series.json.

## Verification

Byte-identical native source fixturebfe3fc402e8ae6cdec74d8b35fa8f50e612f2a8e,
docs/audit/c-reports/CBUG-0062/scenario.c. Its existing profile environment
parameter isolates output. The runner links actual product core objects and
compiles only disposable replacement ring-module objects for the two partial
alternatives, using recorded native CMake flags. No task source or normal core
object is changed for alternatives. Same native data, helpers, seed, assertions,
items and monster across all variants.

| Production variant | Passing cases | Charged left ring |
| --- | --- | --- |
| Original four-carry core | 5/7 | Not activated; enemy retains70HP |
| Ring detection only extended | 5/7 | Not activated; enemy retains70HP |
| Activation only extended | 5/7 | Not activated; enemy retains70HP |
| Both loops extended | 7/7 | Left ring activates; enemy killed |

Affected cases: ready left ring; right ring recharging while left is ready.
All five control logs are byte-identical across all four versions: right ring
ready, both rings ready/right remains preferred, left ring recharging, no ring,
blindness. Corrected left activation starts native recharge98 and temporary
fire protection37; player HP is unchanged. The actual device failure chance
49/1000 remains; this seed's successful activation is not a guarantee for all runs.
The fixture isolates this native attack with constructed items/monster and an
adapter executing actual selected keys. It does not compare every other attack
in a full Borg turn or prove encounter-frequency/player-death/survival benefit.

Normal five-carry native gatee05aedfad:965/965unit +5/5frontend, exit0.16 tooling
controls pass. Earlier swap9/knockout4/preparation9/cold6 native scenarios are
verified on this cumulative core with all original controls/expectations. Thus
35 corrected native scenarios pass; partial-variant assertion failures remain
expected and checked numerically, not hidden by their native process exit0.
No redundant full local gate is added for later docs.

After merge the actual source is rebuilt and run under the same immutable
Warrior seeds11/29/47/cap20000/wall120s/stall30s cohort. No population survival
improvement is assumed. The death pilot's separate original337e50eb source,
seed11 death and pre-death replays remain frozen; later fixes do not rewrite it.

Fresh native reproduction from a clone containing this carry:

```sh
git fetch source-tracker task/cbug-0062-evidence
git worktree add --detach .worktrees/ring-before 337e50eb6c732bf6c908ab0d7c22b12b3a82b7d3
(cd .worktrees/ring-before && flock /tmp/angband-validation.lock python3 tools/verify-native.py)
flock /tmp/angband-validation.lock python3 tools/verify-native.py
flock /tmp/angband-validation.lock python3 tools/verify-rings.py --before-build .worktrees/ring-before/build/native --output artifacts/rings
```

Set up the push-disabled source-tracker remote if absent as in README. The
fixture pin, not moving FETCH_HEAD, is read. Generated logs, builds and binary
variants stay ignored. Source ownership/worktree remains independent. No official
Angband submission or new strategy; locally carried/open +patch-active until
upstream incorporation is rebuilt/verified. Ready bugs pre-empt death tooling.
