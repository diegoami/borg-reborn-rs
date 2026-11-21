# source591: model Mind Blast as 8d8 damage plus confusion (patch 0140)

Consumer adoption of [source 591](https://github.com/diegoami/neo-angband-tracker/issues/591),
source pair [#602](https://github.com/diegoami/neo-angband-tracker/issues/602),
fix `9a9fb3ce8` (branch `task/mindblast-591`), kit `docs/audit/c-reports/mindblast-591` from the
evidence branch `task/mindblast-591-evidence` (generators byte-identical to the fixtures run-state
pin `b40e88f228`; the captured artifacts were added by later evidence-branch commits; the
interpreter `__pycache__` directory is not carried).

## Change

The Mind Blast spell case of `borg_danger()` scored the damage as `spell_power/2+1`, a 3.x-era
figure. In 4.2 Mind Blast is an 8d8 attack with a confusion side effect, so the Borg underrated
casters that carry it: at the measured position the whole Mind Blast term was 9 of a danger 40
dominated by the Wound spell, where the game's 64 plus the confusion term gives 74. Score the
damage as the 8d8 maximum and add the confusion term the other confusion-bearing spells use.

Source `9a9fb3ce8` -> applied `28fec8793` (`cherry-pick -x`), patch `patches/0140-mind-blast-danger.patch`,
scope `src/borg/borg-danger.c` only (6 insertions, 2 deletions; single hunk, applies at a
+23-line offset over the module's carried corrections, byte-wise over the layered result).

## Verification

Before commit `623148199` (the kit commit, `src/` identical to `origin/main`), tip `99f86ebeb`, kit
unchanged (`tools/verify-mindblast591.py`). Native gates green at both (965/965 unit checks and
5/5 frontend tests at the before commit and at the tip). Byte-wise provenance: the source diff
applied to the before module equals the product module, and the `src`/`lib`/build scope of the fix
and of before..tip is the one file. The verifier compiles each kit phase against that phase's own
build tree (the per-phase include-roots pattern from the source589 verifier).

| Mode | Before | After |
| --- | --- | --- |
| `nar` (Nár, the Dwarf, four grids away; 200 HP Warrior, save 39, no confusion protection) | fails: danger 40 (his Wound spell dominates; Mind Blast counted as 9) | passes: danger 74 (Mind Blast 64 + 10 for confusion) |
| `control` (illusionist caster) | passes: danger 600 | passes, same outcome, **byte-identical** |

The committed `before-*.log` / `after-*.log` are the source lane's captures. All four product
logs are **byte-identical** to them (both phases, both modes) and the OUTCOME rows match exactly;
the per-case comparison is recorded in `product-result.json`, and the source's own rows and hashes
are in `receipt.json`, carried unchanged.

## Mode attribution

Per the 2026-11-10 owner rule, each configuration is stated. The monster-spell danger cases sit in
`borg_danger()`, which feeds fight/flee and target choice for every monster in every configuration
— unrestricted (all modes), munchkin/Lunal-only and normal-only — so the changed branch executes
in all of them; the kit exercises it through the production `borg_danger()` with a Mind Blast
caster and a control caster whose danger does not involve the changed case. The per-mode cohort
effect of correctly rating Mind Blast casters is **not measured**, not neutral; the batch
measurement with explicit arms is owed to borg-zoo issue 87. Not measured, per the source: the
save chance against the confusion (the term is added unconditionally here), the fight, deaths, or
how often a Mind Blast caster is met at low save.

Matrix group `mindblast-591` (2 cases) uses the `after-*.log` rows as its baseline and is
byte-identical in both cases. Full matrix at this tree: 118 groups / 365 cases, `ok: true`, zero
failed cases, 299/365 byte-identical; the 26 byte differences are the same known ones as before
this carry — no new difference.
