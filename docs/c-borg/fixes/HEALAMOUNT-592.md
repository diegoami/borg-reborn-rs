# source592: use the fixed heal amounts of Cure Light/Serious/Critical Wounds (patch 0133)

Consumer adoption of [source 592](https://github.com/diegoami/neo-angband-tracker/issues/592),
source pair [#600](https://github.com/diegoami/neo-angband-tracker/issues/600),
fix `65151df9e` (branch `task/healamount-592`), kit `docs/audit/c-reports/healamount-592` from the
evidence branch `task/healamount-592-evidence` (generators byte-identical to the fixtures run-state
pin `5c6a2aa4bc11`; the captured artifacts were added by later evidence-branch commits).

## Change

Before drinking a cure potion, `borg_heal()` checks that the potion will heal enough to cover the
danger it faces, and that the heal is worth at least a third of that danger. It estimated Cure
Light/Serious/Critical Wounds as a percentage of the hit points lost — 15/20/25 %, at least
15/25/30 — and its comments still call them 2d10, 4d10 and 6d10, the 3.x amounts. In 4.2 these
potions heal a fixed 20, 40 and 60 (`object.txt` `HEAL_HP` dice with no percentage part;
`effect_handler_HEAL_HP` heals max(m_bonus% of the hit points lost, base+dice); CLW/CSW/CCW have
no m_bonus). At low hit points the Borg expected too little and left a potion in its pack that
would have covered the danger. Use the fixed amounts; the Potion of Healing estimate (300, or
35 % of the hit points lost) already matches the game and is unchanged.

Source `65151df9e` -> applied `1454dc8bf` (`cherry-pick -x`), patch `patches/0133-heal-amounts.patch`,
scope `src/borg/borg-caution.c` only (8 insertions, 15 deletions). The module already carries
source364/back-away/desperate-stairs/recall-mana/fleeing-guard corrections and the I-001/I-002
improvements; the source hunks apply at a one-line offset and the provenance check below is
byte-wise over the layered result.

## Verification

Before commit `684968350` (the kit commit, `src/` identical to `origin/main`), tip `59cce5864`, kit
unchanged (`tools/verify-healamount592.py`). Native gates green at both (965/965 unit checks and
5/5 frontend tests at the before commit and at the tip). Byte-wise provenance: the source diff
applied to the before module equals the product module, and the `src`/`lib`/build scope of the fix
and of before..tip is the one file.

| Mode | Before | After |
| --- | --- | --- |
| `window` (danger 89, HP 57/103, danger 32 above current HP — the CSW band) | fails: no potion; the Borg reads Word of Recall instead (`quaffs_csw=0`) | passes: "# Quaffing Potions of Cure Serious Wounds." (`quaffs_csw=1`) |
| `control` (danger 89, HP 39/70, danger 50 above current HP — beyond any CSW) | passes: no potion (it cannot cover the danger) | passes, same outcome, **byte-identical** |

In the changed case the potion's real 40 covers the danger (89 < 57 + 40) and is more than a third
of it; the original's 25 fails both tests (89 ≥ 57 + 25, and 25 < 89 / 3). A light cut (timer 5)
bypasses the carried source364 random "heal only sometimes" skip, so the amount test decides alone.

The committed `before-*.log` / `after-*.log` are the source lane's captures. Product logs are
byte-identical to them in the `window` mode (both phases). The `control` mode passes in both
phases with identical STATE/OUTCOME rows and byte-identical before/after product logs, but its
`TRACK` diagnostic line differs from the pristine capture: the capture reads
`# Fleeing (failed to teleport)`, this tree reads `# Fleeing (excessive danger)` — the carried
fleeing-guard correction (source156) changes the flee reason the Borg reports for this position,
not the decision. Carried-layer difference, outcomes identical; the group joins the owed baseline
re-point follow-up on checkpoint1. The per-case comparison is recorded in `product-result.json`,
and the source's own rows and hashes are in `receipt.json`, carried unchanged.

## Mode attribution

Per the 2026-11-10 owner rule, each configuration is stated. `borg_heal()` runs in
`borg_caution()` in every configuration — unrestricted (all modes), munchkin/Lunal-only and
normal-only — whenever a cure potion is considered, so the changed branch executes in all of them;
the kit shows the changed branch executing (the CSW band flips `quaffs_csw`) and records the
control band where no amount change can flip the decision. The per-mode cohort effect of the
corrected amounts is **not measured**, not neutral; the batch measurement with explicit arms is
owed to borg-zoo issue 54. Not exercised by the kit: the CLW and CCW rungs (the same one-line
change) and the high-hit-point direction, where the old percentages overrated the potions; the
source364 random skip still applies to an uncut Borg.

Matrix group `healamount-592` (2 cases) uses the `after-*.log` rows as its baseline; the
`control` TRACK difference above is recorded information, not a failure. Full matrix at this
tree: 111 groups / 348 cases, `ok: true`, zero failed cases, 284/348 byte-identical; the 24 byte
differences are the 23 known ones (13 in the six pre-existing groups, dangerfull-69,
teleotier-176, wandnames-38) plus the one `healamount-592` control TRACK line documented above —
no new unexpected difference.
