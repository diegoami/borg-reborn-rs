# Per-spell projectile classification — consumer44

A blocked boulder sets a flag that leaks into a giant's later lightning area
spells. The existing caster-path correction fixes the open shot, but an actual
mold blocker still causes the original danger code to dismiss those area spells.
Resetting classification per spell restores their assessment. In the native
blocked case the Rogue now spends one Phase Door scroll and loses54HP instead
of59HP. The landing remains exposed; this is not a guaranteed escape or an
established population survival benefit.

Canonical source64/CBUG-0048, discovery borg-rs; reviewed PR277 correction
b223c4f18622883549682f9f997b76fe1f06fc73 imported as
f383220f299d22993bc8b2e0ffa907d966a47823. Patch0009/series retain credit,
source/applied/evidence pins and independent removal unit. The existing cold
protection patch0004 remains in the same module; pristine+0004+0009 reproduce
product borg-danger.c SHA256537f75135363608cf52a9d54d2eb00db381d8cabc0fbeff05a2ccda2c1b6b61d.
Source279 helper remains a separate patch0008; source207 worktree untouched.

## Actual product and interaction proof

Byte-identical fixture0507c24cccd2131710e4e6cd566f901bad94e081,
docs/audit/c-reports/CBUG-0048/scenario.c. Native core objects from eight-carry
before and nine-carry after. Before logical8aff7656 reuses qualified2a99a34d
build; actual commit/binary recorded, src/lib/CMake inputs proven identical.
Diagnostics remove the helper using only a disposable original projection
object compiled with recorded before CMake flags. Ordinary sources/objects do
not change to create alternatives; every other carried correction is preserved.

| Native variant | Passing cases |
| --- | --- |
| Neither path nor per-spell correction |3/5|
| Path only — actual product before |4/5|
| Per-spell only — diagnostic |5/5|
| Both — actual product after |5/5|

Actual incremental affected case is the genuine-blocker storm giant. Clear-path,
patriarch ball, orc direct spell and pure hill-giant **four entire control logs
are identical before/after**. Original standalone clear-path54→0HP benefit
is retained in the neither/per-spell-only comparison; it is not attributed anew
to this incremental carry on the already helper-fixed product.

| Actual helper-fixed product | Before | After |
| --- | --- | --- |
| Clear storm giant |1scroll,0HP lost|Same|
| Mold-blocked storm giant |No escape,59HP lost|1scroll,54HP lost; still hit|
| Patriarch area spell |1scroll,47HP lost|Same|
| Orc direct spell |2mana,no HP loss|Same|
| Pure hill giant |2mana,29HP lost|Same; earlier18→29HP tradeoff retained|

All49 earlier non-geometry fixture cases have byte-identical complete logs.
All six source279 geometry cases still pass with identical geometry fields;
five whole logs match, while the mixed blocked case intentionally changes from
59HP/no scroll to54HP/one scroll under this correction. That interaction is
explicitly asserted, not hidden by claiming every old outcome is unchanged.
Thus55 prior cases and five new fixture cases pass:60 cumulative native cases,
not60 independent gameplay encounters. Normal gatef383220f:965/965unit+5/5frontend;
29 tooling controls pass. No assertion/cap/seed weakening or failed result hidden.

Conditions/limits: constructed level20 Rogue,130HP/121mana,depth38,seed1000,
lit mapped arena, Intelligence/ring, known Phase Door/one scroll, internal age100.
Production danger/escape choose; exact spell/book/item/key sequence validated
and executed natively. One valid native caster spell is scheduled only when
terrain/range permit and with the correct active caster.5%Phase Door failure
retained; recorded attempts succeed. No full AI cast selection/frequency, all
fallback actions, world/energy/UI timing, pursuit/whole fight/death claim.
Different actions change later random consumption; no reset or invented roll.
Danger is a heuristic, not HP prediction. One scroll for a5HP difference and an
exposed landing does not establish uniformly better resource use.

Reproduce from a clone containing this carry:

```sh
git fetch source-tracker task/c-borg-per-spell-projectile task/cbug-0048-evidence
git worktree add --detach .worktrees/per-spell-before 8aff765683493311ca61d66ca1e4f02d241d7a37
(cd .worktrees/per-spell-before && flock /tmp/angband-validation.lock python3 tools/verify-native.py)
flock /tmp/angband-validation.lock python3 tools/verify-native.py
flock /tmp/angband-validation.lock python3 tools/verify-per-spell.py --before-build .worktrees/per-spell-before/build/native --output artifacts/per-spell
```

Configure push-disabled source remote as in README. Actual merged build is
measured separately under unchanged frozen Warrior seeds11/29/47,cap20000,
wall120/stall30. Every run/failure/source/build/binary/data/RNG/calendar identity
and raw result is retained; public Verification/actual merged impact and ignored
durable archive recovery on consumer44. No new discretionary cohort or relabeling
of previous deaths. Carried/open+patch-active awaits upstream incorporation,
rebuild/verification. No official posting, source lane racing or new tactic.
