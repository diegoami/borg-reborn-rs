# source610: model unresisted elemental vulnerability in borg_danger() (patch 0137, stacked on 0136)

Consumer adoption of [source 610](https://github.com/diegoami/neo-angband-tracker/issues/610),
source pair [#617](https://github.com/diegoami/neo-angband-tracker/issues/617),
fix `16a8ac653` (branch `task/vulnmod-610`, **stacked on the #594 correction** `82fd25a54` —
carried here as 0136/PR 517), kit `docs/audit/c-reports/vulnmod-610` from the evidence branch
`task/vulnmod-610-evidence` (generators byte-identical to the fixtures run-state pin
`91685999bb`, whose before-tree already contains the 594 correction).

## Change

After #594, a resistance cancelled by a vulnerability and a never-resisted element look the same
in `BI_RFIRE`, but the game treats them differently: an unresisted elemental vulnerability takes
×4/3 damage, and temporary resistance cancels the vulnerability's multiplier. The 17 element
cases of `borg_danger()` (melee, breath, ball, bolt, beam) applied neither. The fix adds traits
`BI_VFIRE`, `BI_VELEC`, `BI_VCOLD`, `BI_VACID` recording a vulnerability that has no permanent
resistance to cancel, and applies ×4/3 without temporary resistance and no reduction with it.

Source `16a8ac653` -> applied `a817f51c9` (`cherry-pick -x`, three-way over the carried 0136),
patch `patches/0137-vuln-danger-modelling.patch`, scope the three files
`src/borg/borg-danger.c`, `src/borg/borg-trait.c`, `src/borg/borg-trait.h` (+4 enum entries).
The provenance base is the stacked source parent `82fd25a54` (the consumer's 0136 provides that
content); the stacked diff applied to the before modules equals the product modules.

## Verification

Before commit `cb8c51fcb` (the kit commit, `src/` identical to `origin/main` including 0136),
tip `5a5626f0a`, kit unchanged (`tools/verify-vulnmod610.py`). Native gates green at both (965/965
unit checks and 5/5 frontend tests at the before commit and at the tip). The verifier compiles
each kit phase against that phase's own build tree (the per-phase include-roots pattern from the
source589 verifier; this fix adds four enum entries to `borg-trait.h`).

| Mode | Before | After |
| --- | --- | --- |
| `vuln` (game fire res −1, no temp) | fails: the ×4/3 multiplier is not applied (danger 137 = the reference) | passes: 169 against the 137 reference |
| `temp` (temp fire resistance, vulnerability present) | fails: temp resistance wrongly cancels the vulnerability (danger 73 vs reference 137) | passes: 137, the multiplier cancelled with the vulnerability |
| `control` (no vulnerability) | passes: 137 = reference | passes, same outcome, **byte-identical** |

The committed `before-*.log` / `after-*.log` are the source lane's captures. All six product
logs are **byte-identical** to them (both phases, all three modes) and the STATE/OUTCOME rows
match exactly; the per-case comparison is recorded in `product-result.json`, and the source's own
rows and hashes are in `receipt.json`, carried unchanged.

## Mode attribution

Per the 2026-11-10 owner rule, each configuration is stated. The 17 element cases sit in
`borg_danger()`, which feeds fight/flee and target choice for every monster in every
configuration — unrestricted (all modes), munchkin/Lunal-only and normal-only — so the changed
branches execute in all of them; the kit exercises the multiplier, its cancellation by temporary
resistance, and the no-vulnerability control through the production `borg_danger()`. The
per-mode cohort effect of correctly rating vulnerability damage is **not measured**, not neutral;
the batch measurement with explicit arms is owed to borg-zoo issue 54.

Matrix group `vulnmod-610` (3 cases) uses the `after-*.log` rows as its baseline and is
byte-identical in all three cases. Full matrix at this tree: 115 groups / 359 cases, `ok: true`,
zero failed cases, 295/359 byte-identical; the 24 byte differences are the same known ones as
before this carry — no new difference.
