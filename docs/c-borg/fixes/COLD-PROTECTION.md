# Use cold protection for cold bites — consumer17

An acid-protected Rogue without cold protection stays beside a snow troll and
loses21HP because the Borg credits acid protection against cold bites. Correcting
the protection lookup makes it Phase Door away for2mana and avoid the scheduled
attack. **The reverse case is a tradeoff:** with cold protection alone, the
correction stays, saving2mana but taking20HP instead of escaping unharmed. The
fix makes the input mapping correct; it does not establish universal safety.

Canonical [source26/CBUG-0004](https://github.com/diegoami/neo-angband-tracker/issues/26),
reviewed C-only PR267 d504eb3d272ce96c0de2c00eba757a251b551334, discovery
borg-rs. Imported with cherry-pick -x as2827b28fc; original patch/removal unit
and source/applied pins in patches/series.json. Final borg-danger.c SHA256
`bad414fee0b7b1cedf694f0726ad4b10bb97bf3717032e80472777eb89adbe58`
matches the source proposal. One identifier reads res_cold instead of res_acid
in cold-melee danger. No RNG call, damage rule, escape threshold or strategy
change. This consumer task does not modify the source lane/worktree.

## Verification

Byte-identical native fixture:
`b0b0fa2471a11e91838bcbc3d980f4b04f8c6b01`,
`docs/audit/c-reports/CBUG-0004/scenario.c`.
The existing CBUG0004_PROFILE parameter isolates generated files; no fixture
source edits. Actual product core objects before/after use the same test helpers
and native data. Before includes source82,14 and110; this isolates cold protection.

| Protection / attacker | Before HP lost / resources | After HP lost / resources |
| --- | --- | --- |
| Acid only / snow troll | 21HP /0mana | 0HP /2mana |
| Cold only / snow troll | 0HP /2mana | 20HP /0mana |
| Neither / snow troll | 0HP /2mana | Identical |
| Both / snow troll | 20HP /0mana | Identical |
| Acid only / scorpion with no cold blow | 0HP /1Phase Door scroll | Identical |
| Cold only / same scorpion | 0HP /1Phase Door scroll | Identical |

Two intended-choice checks fail before/pass after, **6/6 corrected cases pass**,
**4 complete control logs byte-identical**. The verification explicitly asserts
both the protective result and the less-favorable HP counterexample. Existing
escape threshold104 remains unchanged. Acid-only danger52→116 crosses it;
cold-only116→52 stays below. Those scores are not measured HP damage. Normal
native attacks separately establish the HP result; scroll escapes are not free.

Human Rogue level20,130native HP/121mana,depth28, known Phase Door spell/scroll,
legal +6Int ring and lit40x40arena are constructed. Timed protections, items,
identification and monster positions are declared fixture setup. Production
borg_escape chooses actions; an adapter validates/executes its actual spell/book
or scroll commands. One ordinary native melee action is scheduled only if the
monster remains adjacent. Hits/damage are not forced. Internal preparation age100
passes an existing cooldown;100world turns are not run. Phase Door's5% failure
chance remains; seed1000 succeeds here. No full caution loop, pursuit, alternate
fallback moves, world/UI timing, encounter frequency or player death is proved.

Native damage takes the larger of physical/elemental components, and the troll
also claws. Cold protection therefore does not remove all HP loss. Accepting
20HP to save2mana is the existing policy's tradeoff, not proof of a new optimal
strategy. Death-driven suggestions about it belong in evidence-backed issues
first; the current patch does not tune the threshold.

Normal native gate atb48f954d7:965/965unit +5/5frontend, exit0. Earlier swap,
knockout and preparation scenarios are rerun on this four-carry core. All16
measurement/tooling controls remain applicable; later docs do not alter runtime
or justify another full local gate. Actual merged source is measured on the
unchanged3-seed/20000-turn Warrior cohort and reported on consumer17; no favorable
cohort expansion or survival effect is assumed.

Fresh reproduction from this carry (source-tracker push-disabled as in README):

```sh
git fetch source-tracker task/cbug-0004-evidence
git worktree add --detach .worktrees/cold-before 8208d7886208414c3e6ca2df4885175855f34011
(cd .worktrees/cold-before && flock /tmp/angband-validation.lock python3 tools/verify-native.py)
flock /tmp/angband-validation.lock python3 tools/verify-native.py
flock /tmp/angband-validation.lock python3 tools/verify-cold-protection.py --before-build .worktrees/cold-before/build/native --output artifacts/cold-protection
```

Source fixture is immutable, not moving FETCH_HEAD. Generated logs/results/builds
stay ignored, with persistent evidence/hashes at the consumer checkpoint.
Consumer17 stays carried/open +patch-active until actual upstream incorporation
is rebuilt/verified. No official submission was made by the agent. This ready
bug takes priority over discretionary death-analysis implementation.
