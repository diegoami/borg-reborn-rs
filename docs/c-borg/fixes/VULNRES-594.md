# source594: let a vulnerability cancel the matching permanent resistance (patch 0136)

Consumer adoption of [source 594](https://github.com/diegoami/neo-angband-tracker/issues/594),
source pair [#608](https://github.com/diegoami/neo-angband-tracker/issues/608),
fix `82fd25a54` (branch `task/vulnres-594`), kit `docs/audit/c-reports/vulnres-594` from the
evidence branch `task/vulnres-594-evidence` (generators byte-identical to the fixtures run-state
pin `6ec100d75`; the captured artifacts were added by later evidence-branch commits).

## Change

`calc_bonuses()` (player-calcs.c) records any item's `res_level == -1` as a vulnerability and
then, after combining all items, lowers the level by one more unless the player is immune; timed
resistance is added after that. The burning-up curse is `RES_FIRE[-1] | RES_COLD[1]` (curse.txt).
The Borg read the vulnerability traits (`BI_CRSFVULN`, `BI_CRSCVULN`, `BI_CRSEVULN`,
`BI_CRSAVULN`) only in `borg_power()` and never cleared a resistance trait when the matching
vulnerability was present, so a rFire armour combined with a fire-vulnerable shield counted as
fire-resistant. Clear the four elemental resistance traits when the matching vulnerability trait
is set.

Source `82fd25a54` -> applied `f6cf2a0b1` (`cherry-pick -x`), patch `patches/0136-vuln-cancels-resistance.patch`,
scope `src/borg/borg-trait.c` only (14 insertions; applies at a −4-line offset over the module's
carried corrections, and the provenance check below is byte-wise over the layered result).

## Verification

Before commit `2126110d4` (the kit commit, `src/` identical to `origin/main`), tip `33a56945a`, kit
unchanged (`tools/verify-vulnres594.py`). Native gates green at both (965/965 unit checks and 5/5
frontend tests at the before commit and at the tip). Byte-wise provenance: the source diff applied
to the before module equals the product module, and the `src`/`lib`/build scope of the fix and of
before..tip is the one file. The verifier compiles each kit phase against that phase's own build
tree (the per-phase include-roots pattern established by the source589 verifier).

| Mode | Before | After |
| --- | --- | --- |
| `vuln` (rFire armour + fire-vulnerable shield) | fails: the vulnerability does not cancel rFire (`borg_rfire=1`, `borg_fire_vuln=1`, danger 72) | passes: rFire cancelled (`borg_rfire=0`, danger 134) |
| `control` (genuinely fire-resistant) | passes: rFire 1, danger 72 | passes, same outcome, **byte-identical** |

The committed `before-*.log` / `after-*.log` are the source lane's captures. All four product
logs are **byte-identical** to them (both phases, both modes) and the STATE/OUTCOME rows match
exactly; the per-case comparison is recorded in `product-result.json`, and the source's own rows
and hashes are in `receipt.json`, carried unchanged.

## Mode attribution

Per the 2026-11-10 owner rule, each configuration is stated. The corrected trait computation runs
in `borg_notice()` in every configuration — unrestricted (all modes), munchkin/Lunal-only and
normal-only — whenever gear changes, so the changed branch executes in all of them; the kit shows
it executing (the vuln mode flips `borg_rfire` 1 → 0 and the fire danger 72 → 134) and records
the genuinely-resistant control where the corrected branch does not fire. The per-mode cohort
effect of correctly rating cursed-vulnerability gear is **not measured**, not neutral; the batch
measurement with explicit arms is owed to borg-zoo issue 54.

The stacked source610 correction (unresisted vulnerability ×4/3 in `borg_danger()`) is a separate
follow-up unit carried after this one, per the source pair's ordering.

Matrix group `vulnres-594` (2 cases) uses the `after-*.log` rows as its baseline and is
byte-identical in both cases. Full matrix at this tree: 114 groups / 356 cases, `ok: true`, zero
failed cases, 292/356 byte-identical; the 24 byte differences are the same known ones as before
this carry — no new difference.
