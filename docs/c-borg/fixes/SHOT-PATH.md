# Caster-origin projectile path — consumer40

A clear-path hill giant is counted as obstructing its own boulder; native danger
is zero despite an actual18HP hit. Ignoring the starting caster repairs that
path contract while preserving real interior monsters, walls and endpoints.
The resulting escape is **not uniformly safer**: this hill-giant case spends
2mana and loses29HP instead of18. A clear-path storm giant instead costs one
scroll and avoids54HP. Both outcomes remain asserted and published.

Canonical native source279/PR281, independent native C discovery while validating
source64 (no reassigned historical CBUG ID). Reviewed source
1bce07d8080b520589c292c793cc0b95ef9df182 imported with attribution as
2a99a34d174edfe257c25a383c8f952eed71a3b6. Original patch0008/series ledger is an
independent removal unit. Product borg-projection.c matches the reviewed source:
SHA256918ac82bb1f0ed197adfc07205a2d185ac14bcfd25f8357d852d504d54e894e2.
Pristine+patch0008 round-trip reproduces it. Source207 worktree stays untouched.

## Actual product verification

Byte-identical fixturecef0b118e02976a3384d4c348ba4cd0210c1f492,
docs/audit/c-reports/shot-path-279/scenario.c. Actual seven-carry before and
eight-carry after core objects, with the original seed/assertions/data/profile.
Before logical snapshot9929eb4e reuses the qualified3d12dc53 native build;
actual build commit/binary hash recorded, src/lib/CMake inputs proven identical.
The independent per-spell64/PR277 alternative b223c4f1 compiles only a disposable
danger object with recorded before CMake flags; the existing cold carry is
preserved. Ordinary sources/core objects are never mutated for alternatives.

| Variant | Intended path cases passing |
| --- | --- |
| Seven carries, no path/per-spell fix |2/6|
| Independent per-spell correction only |2/6|
| Helper and per-spell correction |6/6|
| Helper-only product correction |6/6|

The four original caster-origin failures complete after native actions/outcomes;
exit0 alone is not assertion success. Every corrected case checks both caster
and player directions, source endpoint, wall and beyond-wall target. Actual
mold endpoints stay valid; absent mold is recorded−1/not queried. Two entire
intermediate-mold logs are identical before/helper-after. Four control
DECISION/FRAME/OUTCOME sequences are identical (both mold cases, patriarch ball,
orc shaman direct spell); some path fields change in the last two, so their
whole logs are not falsely called identical.

| Scenario | Before | Helper-only after |
| --- | --- | --- |
| Hill giant, clear |No escape,18HP lost|2mana,escape,29HP lost|
| Storm giant, clear |No escape,54HP lost|1scroll,behind wall,0HP lost|
| Hill giant, actual mold |No escape,0HP lost|Same|
| Storm giant, actual mold |No escape,59HP lost|Same; per-spell defect still present|
| Patriarch ball |1scroll,47HP lost|Same|
| Orc shaman direct |2mana,no scheduled spell/HP loss|Same|

Native eight-carry gate2a99a34d:965/965unit+5/5frontend;29 tooling tests.
All49 earlier corrected native scenarios have byte-identical complete logs on
this core;55 cumulative corrected cases. First runner attempt completed24 native
cases but failed geometry parsing because the unit runner prefixes GEOMETRY
with its test name. It is retained, not counted as verified. Matching the
original source parser fixes the invocation; no fixture/assertion was weakened.

Conditions/limits: constructed level20 Rogue,130HP/121mana,depth38,seed1000,
lit mapped arena, Intelligence/ring, known Phase Door/one scroll, internal clock100.
Production danger/escape choose; exact keys/book/item native commands execute.
One valid caster spell is scheduled only if real terrain/range permit, in actual
active-monster context. Native damage/saves/movement/resources computed;5%Phase
Door failure retained, shown attempts succeed. No full AI cast choice/frequency,
all fallbacks/UI/energy/world timing, pursuit/whole fight/death/frequency proof.
Danger192 is a heuristic, not HP prediction. Different actions naturally consume
different RNG; no reset to force identical subsequent encounters.

Fresh reproduction:

```sh
git fetch source-tracker task/c-borg-caster-shot-origin task/shot-path-279-evidence task/c-borg-per-spell-projectile
git worktree add --detach .worktrees/shot-before 9929eb4eb16412ef2415a5711a3eecab92fa1288
(cd .worktrees/shot-before && flock /tmp/angband-validation.lock python3 tools/verify-native.py)
flock /tmp/angband-validation.lock python3 tools/verify-native.py
flock /tmp/angband-validation.lock python3 tools/verify-shot-path.py --before-build .worktrees/shot-before/build/native --output artifacts/shot-path
```

Push-disabled source remote setup in README. After merging, rebuild the actual
merge and measure unchanged frozen Warrior seeds11/29/47,cap20000/wall120/stall30;
paired starts do not guarantee identical later encounters. Results/every attempt
and persistent ignored archive recovery on consumer40; no benefit assumed.
Independent per-spell64 remains diagnostic only, not silently adopted here.
Old death/build identities and milestone2's seven-carry snapshot remain frozen.
Carried/open+patch-active awaits actual upstream incorporation/rebuild/verification.
