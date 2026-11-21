# Cast the selected preparation spell once — consumer15

The Borg chooses haste but casts regeneration first, spending20 extra mana.
If regeneration wins instead, it casts it twice. The correction keeps scoring
free of cast side effects and executes only the chosen spell once. This fixes
an extra action/resource expenditure; a second regeneration also extends its
duration, so the evidence does not show that the extra cast is always harmful.

Canonical [source110/CBUG-0094](https://github.com/diegoami/neo-angband-tracker/issues/110),
source [PR264](https://github.com/diegoami/neo-angband-tracker/pull/264), discovery
borg-rs. Reviewed C-only31a042d00581ed914d30f17cca1ed0e70e99d21f was imported
with cherry-pick -x as0e76ee35138b309ceb3ed43a6427f2baa42e6d1f, separately from
the two earlier carries. The final borg-fight-perm.c SHA256
`7ac6bb54192d2dae6a4dfe82d93c3ba0c7d3f7b796a60cede58190010582bba7`
matches that source blob. Four lines add the existing simulation-score return;
no other production source changes, eligibility/selection thresholds or rules.
The original patch and source/applied pins are in patches/series.json.

## Verification

Unchanged, byte-identical native fixture:
`a19c8733c0dfa8139b04eecd8117429ec5f55cd6`,
`docs/audit/c-reports/CBUG-0094/scenario.c`.
Its existing CBUG0094_PROFILE parameter isolates output; no fixture source
adaptation is needed. The consumer runner links actual product core objects
before and after, using the same native test helpers/data. Before includes
source82 and source14; this comparison isolates the regeneration correction.

| Selected spell | Before actual casts | Before mana/energy | After casts | After mana/energy |
| --- | --- | --- | --- | --- |
| Regeneration | Regeneration twice | 40mana /200action energy | Regeneration once | 20mana /100action energy |
| Haste | Regeneration then haste | 32mana /200action energy | Haste only | 12mana /100action energy |

Both affected cases fail before/pass after; **9/9 after pass**, **7 complete
control logs byte-identical**: haste-only, regeneration already active,
regeneration unlearned, blindness, confusion, town and insufficient spare mana.
All18 cases complete native spell execution; assertions and numeric outcomes
are checked because the native unit process can exit0 after assertion failure.
Starts401mana: haste case ends369before/389after, regeneration361before/381after.

The fixture constructs a native Human Druid, lit arena, level50, legal +6Wisdom
amulet and spell book/knowledge, full HP/SP, seed1000. The real Borg selects and
queues keys; an adapter decodes its actual book/offset commands and calls native
casting. Native code charges mana/energy and applies effects. No substituted
spell, forced success or damage result. There is no world processing between
casts: energy is native action cost, **not elapsed world turns or observed extra
enemy attacks**. Duration/RNG differ after removal of the extra cast. No full
normal-run frequency, combat-damage, player-death or survival claim is made.

Normal native gate on4dcc5ad0d: **965/965 unit +5/5 frontend**, exit0. All16
measurement/tooling controls pass. Earlier9-case safer-swap and4-case knockout
native regressions are rerun against the three-carry product core, retaining
their original before/fixed expectations and unchanged controls. Later docs do
not change tested runtime; no redundant full local gate is introduced.

After merge, rebuild the actual product source and run the **unchanged**
Warrior seeds11/29/47/cap20000/wall120s/stall30s cohort. Record the outcome rather
than expecting a survival effect: those shallow non-caster runs cannot appraise
this level50 Druid case. Keep prospective caster/death exploration separate from
this frozen regression cohort. Actual merged pin/results are posted on issue15.

Fresh replay from a consumer clone containing this carry and source objects:

```sh
git fetch source-tracker task/cbug-0094-evidence
git worktree add --detach .worktrees/preparation-before d4bfdce9d4782cf83994ad52de707a52d1f31018
(cd .worktrees/preparation-before && flock /tmp/angband-validation.lock python3 tools/verify-native.py)
flock /tmp/angband-validation.lock python3 tools/verify-native.py
flock /tmp/angband-validation.lock python3 tools/verify-preparation.py --before-build .worktrees/preparation-before/build/native --output artifacts/preparation
```

Set up the push-disabled source-tracker remote first if absent, as in the
measurement README. The runner reads the immutable evidence commit, not a moving
FETCH_HEAD. Generated runs/logs/builds stay ignored. Canonical source ownership
remains separate; the source worker's26 investigation is untouched. No official
Angband submission has occurred. Consumer15 stays carried/open +patch-active
until actual upstream resolution is incorporated, rebuilt and verified.

This ready verified bug was carried before implementing the discretionary
DEATH-ANALYSIS-PLAN.md fallback, following the owner's bugs-first instruction.
