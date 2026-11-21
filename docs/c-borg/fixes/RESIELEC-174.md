# source174: name the electricity resistance in the resist-electricity note (patch 0143)

Consumer adoption of [source 174](https://github.com/diegoami/neo-angband-tracker/issues/174),
source pair [#626](https://github.com/diegoami/neo-angband-tracker/issues/626),
fix `d7e2377dc` (branch `task/resielec-174`), kit `docs/audit/c-reports/resielec-174` from the
evidence branch `task/resielec-174-evidence` (generators byte-identical to the fixtures run-state
pin `ee22d19ebb`; the captured artifacts were added by later evidence-branch commits; the
interpreter `__pycache__` directory is not carried).

## Change

The `borg_defend_aux_resist_e()` acting pass notes "# Attempting to cast RAcid" — a copy of the
acid routine's text one function above — so the log names the wrong resistance every time the Borg
prepares its electricity defence. The sibling routines name their own element (RFire, RCold, RAcid,
RPois). Name the resistance actually attempted (RElec). The queued keys are identical in both
variants.

Source `d7e2377dc` -> applied `2d576a496` (`cherry-pick -x`), patch `patches/0143-resielec-note.patch`,
scope `src/borg/borg-fight-defend.c` only (1 insertion, 1 deletion; applies with no offset — the
module's carried corrections sit elsewhere).

## Verification

Before commit `6b9fc2def` (the kit commit, `src/` identical to `origin/main`), tip `b23ef01a2`, kit
unchanged (`tools/verify-resielec174.py`). Native gates green at both (965/965 unit checks and 5/5
frontend tests at the before commit and at the tip). Byte-wise provenance: the source diff applied
to the before module equals the product module, and the `src`/`lib`/build scope of the fix and of
before..tip is the one file. The verifier compiles each kit phase against that phase's own build
tree (the per-phase include-roots pattern from the source589 verifier).

| Mode | Before | After |
| --- | --- | --- |
| `elec` (Ring of Lightning, awake energy hound adjacent) | fails: the acting pass writes "# Attempting to cast RAcid" (`note_acid=1`, `note_elec=0`) | passes: "# Attempting to cast RElec" (`note_elec=1`, `note_acid=0`), identical queued keys |
| `control` (plain melee — no improvement found, no note written) | passes | passes, same outcome, **byte-identical** |

The committed `before-*.log` / `after-*.log` are the source lane's captures. All four product
logs are **byte-identical** to them (both phases, both modes) and the STATE/OUTCOME rows match
exactly; the per-case comparison is recorded in `product-result.json`, and the source's own rows
and hashes are in `receipt.json`, carried unchanged.

## Mode attribution

Per the 2026-11-10 owner rule, each configuration is stated. The changed line is the acting pass's
log note in the electricity-defence routine, reached in every configuration — unrestricted (all
modes), munchkin/Lunal-only and normal-only — whenever that defence is chosen; the kit shows it
executing (the note text flips while the queued keys stay identical). Because the one-token change
alters log text only, the compiled decisions and queued keys are identical in both variants, so no
configuration can observe a behaviour difference: the per-mode effect is **nil by construction**,
recorded as such rather than measured (the kit's identical-keys evidence is the demonstration, not
a cohort run).

Matrix group `resielec-174` (2 cases) uses the `after-*.log` rows as its baseline and is
byte-identical in both cases. Full matrix at this tree: 120 groups / 369 cases, `ok: true`, zero
failed cases, 317/369 byte-identical; the 12 byte differences are the declared intentional
transitions (dispel-248, vampire-257, dispel259, fleeheal-408, preparation) — no new difference.
