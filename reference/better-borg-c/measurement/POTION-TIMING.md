# Healing-potion hesitation after recent drinks — consumer31

A wounded, confused Warrior with seven Cure Critical Wounds potions declines
a drink five Borg turns after the previous one. The corrected native helper
uses a real potion:321→381HP, confusion10→0, seven→six potions. A first-ever
drink at internal turn100 is also restored. This fixes an elapsed-time condition,
not healing amounts, emergency callers, random thresholds or a new tactic.

Canonical source77/CBUG-0061, discovery borg-rs; source PR271 correction
cedf25eb790f2ff7c7c161cd1612b18ed99fb35e imported with attribution as
5e9417be42cb1401708a7a37ec1b13641bc04b83. The independent source207 lane is
untouched. Original patch0006/series.json retain provenance and removal unit.
The cumulative borg-item-use.c includes the earlier ring correction as well;
SHA2562d8da67d6140167531e9a1a953845492b052e272c004b003917ba4a07510f294.
Applying ring0005 then potion0006 to pristine exactly reproduces that module.

## Actual product verification

Byte-identical fixturecce1d5d92079e78e939126d3cf35e24121442734,
docs/audit/c-reports/CBUG-0061/scenario.c; its existing profile environment
isolates disposable output. Actual five-carry before and six-carry after core
objects are linked. The incomplete comparison-only alternative replaces just
the module object, compiled with the recorded before CMake flags, without
mutating ordinary source/build objects.

| Version | Passing cases | Long gap | First drink at internal clock1 |
| --- | --- | --- | --- |
| Five-carry before | 5/9 | Refuses, stays injured/confused | Drinks |
| Comparison reversal only | 8/9 | Drinks/heals | Incorrectly refuses |
| Complete correction | 9/9 | Drinks/heals | Drinks |

Five complete before/after control logs are identical: first drink at clock1,
same-clock requests, explicit emergency bypass, one potion remaining and empty
inventory. Original four expected-choice failures complete normally; process
exit0 alone is not accepted as a passing assertion.

The **recent-drink tradeoff** is retained at elapsed1 and4: before drinks and
heals60HP/clears confusion; after conserves a potion and remains321HP/confused.
The existing same-clock exclusion/reserve policy and randint1(100)<75 test
remain. Different eligibility naturally changes subsequent RNG consumption.
This is not a universal safety or survival improvement.

Native fixture conditions: seed1000, constructed level40 Human Warrior,
depth30, max421HP, known CCW potions, lit arena. Controlled injuries and an actual
initial bypass drink establish the history; Borg internal clock advances
isolate elapsed time, not simulated intervening world turns. The first-use
sentinel edge does not imply an ordinary level40 birth at turn1. The adapter
validates queued q/item keys and executes do_cmd_quaff_potion on that item.
No full healing ranking, monster attacks, world/UI timing or death is simulated.

Six-carry normal gate at5e9417be:965/965unit +5/5frontend;29 tooling tests pass.
All35 prior corrected scenarios rerun on this core and have byte-identical
complete logs to their prior five-carry results (swap9, knockout4, preparation9,
cold6, rings7). Thus44 corrected native scenarios pass. The original first
scenario attempt failed before gameplay with missing constants.txt because of
the working directory; retained as harness evidence. Reusing the actual native
game directory fixes that invocation without changing the fixture/assertions.

Fresh reproduction from a clone containing this carry:

```sh
git fetch source-tracker task/c-borg-healing-potion-timing task/cbug-0061-evidence
git worktree add --detach .worktrees/potion-before 24ba2be9e445f5f08e322ae97f65e5bdd3b4878b
(cd .worktrees/potion-before && flock /tmp/angband-validation.lock python3 tools/verify-native.py)
flock /tmp/angband-validation.lock python3 tools/verify-native.py
flock /tmp/angband-validation.lock python3 tools/verify-potion-timing.py --before-build .worktrees/potion-before/build/native --output artifacts/potion-timing
```

Configure the push-disabled source-tracker remote as in README. Immutable fixture
pins are used rather than moving FETCH_HEAD. The actual merged45eac63bd build completed the unchanged frozen
seeds11/29/47/cap20000/wall120/stall30 comparison.
All generated output stays ignored; actual results and persistent recovery are
published on consumer31. No new cohort is selected to obtain a favorable result.
Locally carried/open+patch-active awaits actual upstream incorporation/rebuild.

## Merged-source v1 results

Actual merge45eac63bd9f7cabe3c4ef1607703324ca8dae0fc was rebuilt and run.
[Machine-readable outcomes](potion-timing-v1.json) retain all three cases and
all controlled case outcomes. Exact-head hosted42219307029 numerical logs have
29 tooling tests,965/965unit+5/5frontend; merged-main42219457629 also succeeded.

All three v1 starts match byte-for-byte and survive the cap, with no death,
completion, stall, crash or harness failure. Full traces now differ: the changed
condition changes whether private Borg RNG is drawn, even before unavailable
potions are rejected. Later encounters are therefore not held identical.
Seeds11/29 retain level3 and maxdepth1/3; seed47 reaches **level2 versus4**, at
unchanged maxdepth2. Preserve that unfavorable progress result. Independent n=3;
no survival improvement, speedup or claim of no regression. Deep-impact-v2 on
issue33 is separately prospectively bounded, not a replacement for this cohort.
