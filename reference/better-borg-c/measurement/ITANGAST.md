# Recognize Itangast without a false unseen attacker — consumer36

A Warrior already sees/tracks Itangast the Fire Drake. The old two-character
pronoun comparison treats that name as unseen 'It'; the full native defense
chooser consumes one True Seeing potion. Exact case-insensitive whole-word
matching recognizes the dragon and retains both potions. Actual unseen native
descriptors Something/It/it still request and consume a potion.

Canonical source102/CBUG-0086, discovery borg-rs. Reviewed source PR273
cad765806eed6431642ab02b53240d7938470c21 imported with attribution as
3d12dc5306be7d032332f3a9a6e6732261b432c0. Separate patch0007 and series.json
retain source/applied/evidence provenance and independent removal unit.
The resulting borg-flow-kill.c SHA256 matches the reviewed source:
0489bc679403e0fc497ce982567b9fc8f5b782f13914e172b17f066765dd3897.
Applying patch0007 to pristine reproduces it exactly; source207 worktree untouched.

## Actual native product proof

Byte-identical fixtureb61f6b611600fdbc4c6370fff3ac2835ee77017f,
docs/audit/c-reports/CBUG-0086/scenario.c, links actual six-carry before and
seven-carry after core objects. Its existing profile parameter isolates output.
All five cases complete; actual assertions show **4/5 before,5/5 after**.
Four entire control logs are byte-identical: visible ogre and actual invisible
ghost descriptions Something/It/it, obtained through native monster_desc.
No invented name string, substitute detection score or copied decision logic.

| Visible Itangast case | Before | After |
| --- | --- | --- |
| Located known monster index |0|1|
| False unseen timestamp |100|0|
| Defense/potion requested |yes|no|
| Potions consumed |1|0|
| Temporary see-invisible |60|0|
| Player HP |421|421|

Native use of the exact selected potion is validated/executed; ghost controls
retain duration61 and potion use in both versions. Danger488/eligibility limit842
stay the same in the named case. Actual gate3d12dc53:965/965unit+5/5frontend;
29 tooling tests. All44 prior corrected scenarios rerun on this core with
byte-identical complete logs (swap9,knockout4,preparation9,cold6,rings7,potion9).
Thus49 cumulative corrected native scenarios pass. No redundant full gate for
subsequent documentation-only merges.

Limits: constructed native level40 Human Warrior/depth52/seed1000,lit arena,
known two True Seeing potions, right Ring of Flames, temporary fire protection50,
visible already-observed dragon six squares away. That protection matters:
an unprotected exploratory setup's danger2000 suppresses defense entirely.
The fixture calls production name lookup and full defense chooser, but not the
entire message parser/caution loop. Native potion command adapter validates
keys/item and uses do_cmd_quaff_potion. No monster attack, damage, world timing
or death is simulated. Future benefit of extra sight is untested; resource
conservation here is not a general survival or safety claim.

Reproduction from a clone containing the carry:

```sh
git fetch source-tracker task/c-borg-exact-unseen-name task/cbug-0086-evidence
git worktree add --detach .worktrees/itangast-before 45eac63bd9f7cabe3c4ef1607703324ca8dae0fc
(cd .worktrees/itangast-before && flock /tmp/angband-validation.lock python3 tools/verify-native.py)
flock /tmp/angband-validation.lock python3 tools/verify-native.py
flock /tmp/angband-validation.lock python3 tools/verify-itangast.py --before-build .worktrees/itangast-before/build/native --output artifacts/itangast
```

Use push-disabled source-tracker remote setup in README; immutable fixture pin
is used, not moving FETCH_HEAD. Native game working directory supplies data.
After merging, rebuild the actual merge and run the unchanged frozen Warrior
seeds11/29/47/cap20000/wall120/stall30 against the prior six-carry cohort. There
is no expected Itangast encounter in that shallow cohort and no population
benefit assumed. Earlier deep results/death identities stay frozen on their
actual versions. All generated runs/saves/builds remain ignored; full actual
postmerge results and persistent evidence receipt belong to consumer36.
Locally carried/open+patch-active only after verification; upstream incorporation
and rebuild/verification are needed for fixed/close. No official posting/tactics.

## Actual merged-source cohort

Merge2675d198e4c3fcb1c69ee63ef25a2b6ac27ab96f was rebuilt and measured under
unchanged frozen v1. [Every case/run summary](itangast-v1.json) records3/3cap
survivors,0deaths/completions/stalls/timeouts/crashes/harness failures. All three
starting saves and complete traces match the prior **six-carry45eac63bd** build,
not the pristine baseline. Levels3/3/2,max depths1/3/2. Earlier potion-related
private-RNG divergence and unfavorable seed47 level4→2 at cap20000 remain.
Independent n=3, shallow cohort cannot establish Itangast encounter benefit;
no survival improvement or runtime speedup claimed. Exact PR head hosted
42221586833 passed29 tooling/965unit/5frontend, numerical logs inspected.
