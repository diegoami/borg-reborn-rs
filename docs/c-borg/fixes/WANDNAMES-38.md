# source38: look up the Confuse Monster and Lightning Balls wands by their names (patch 0131)

Consumer adoption of [source 38](https://github.com/diegoami/neo-angband-tracker/issues/38) (CBUG-0022),
source pair [#584](https://github.com/diegoami/neo-angband-tracker/issues/584),
fix `f9f0384a6` (branch `task/wandnames-38`), kit `docs/audit/c-reports/wandnames-38` from the
evidence branch `task/wandnames-38-evidence` (generators byte-identical to the fixtures run-state
pin `460eddbd46b1`; the captured artifacts were added by later evidence-branch commits).

## Change

`borg_init_item_val()` looked up `sv_wand_confuse_monster` as "Scare Monster" and
`sv_wand_elec_ball` as "Lightning Bolts", copies of the lines above them. The attack chooser
therefore offered a Scare Monster wand as a confusion bolt and a Lightning Bolts wand (a 6d6 beam,
about 21 average damage) as a 64-damage radius-2 ball, and never used a real Confuse Monster or
Lightning Balls wand. Use the wands' own names.

Source `f9f0384a6` -> applied `d3e089107` (`cherry-pick -x`), patch `patches/0131-wand-names.patch`,
scope `src/borg/borg-item-val.c` only (two lines; the module carried no earlier consumer patches).

## Verification

Before commit `0775b3c40` (the kit commit, `src/` identical to `origin/main`), tip `cc6cb96ba`, kit
unchanged (`tools/verify-wandnames38.py`). Native gates green at both (965/965 unit checks and 5/5
frontend tests at the before commit and at the tip). Byte-wise provenance: the source diff applied
to the before module equals the product module, and the `src`/`lib`/build scope of the fix and of
before..tip is the one file.

| Mode | Before | After |
| --- | --- | --- |
| `bolts-vs-frost` | fails: the Lightning Bolts wand is also scored as a ball (`elec_ball=1`, `chose_elec_ball=1`) and chosen over Frost Bolts | passes: ball score 0, Frost Bolts chosen (`chose_cold=1`), **byte-identical to the source capture** |
| `balls-only` | fails: the real Lightning Balls wand is not recognised, no attack (`attacked=0`) | passes: scored 59 and chosen (`attacked=1`, `elec_ball=1`, `chose_elec_ball=1`), **byte-identical to the source capture** |
| `confuse-only` | fails: the real Confuse Monster wand is not recognised (`attacked=0`) | passes: scored as the confusion attack, not zapped (`attacked=0` everywhere) |
| `scare-only` | fails: the Scare Monster wand is also scored as a confusion bolt | passes: scored only as a fear bolt |
| `frost-only` (control) | passes: Frost Bolts chosen | passes, same outcome, **byte-identical** between the phases and to the source capture |

The committed `before-*.log` / `after-*.log` are the source lane's captures. Product logs are
byte-identical to them in every mode except the confusion/fear VALUE diagnostic line in
`scare-only` (both phases) and `confuse-only` (after phase only): the captures record
`VALUE fear=-6` / `VALUE confuse=-6`, the product records `-5`. The difference is in the carried
layer, not this fix — the pristine base scores -6, this tree scores -5 wherever that scoring path
executes, the OUTCOME rows match the captures exactly in every mode and phase, and the
before/after product logs match each other. This is the recorded dangerfull-69/teleotier-176
pattern; the group joins the owed baseline re-point follow-up on checkpoint1. The full verifier
report is `product-result.json`; the source's own rows and hashes are in `receipt.json`, carried
unchanged. Two verifier-constant mistakes were retained and corrected in `cc6cb96ba` without
touching any product assertion.

## Mode attribution

Per the 2026-11-10 owner rule, each configuration is stated. The two corrected lookups run in
`borg_init_item_val()` at every start-up, in every configuration that runs the Borg — unrestricted
(all modes), munchkin/Lunal-only and normal-only — so the changed branch executes in all of them
(the kit shows the corrected svals reaching the production chooser in all five scenarios). The
per-mode cohort effect of the corrected wand scoring is **not measured**, not neutral; the batch
measurement with explicit arms is owed to borg-zoo issue 54.

Matrix group `wandnames-38` (5 cases) uses the `after-*.log` rows as its baseline; the
`scare-only` and `confuse-only` byte differences above are recorded information, not failures.
Full matrix at this tree: 109 groups / 343 cases, `ok: true`, zero failed cases, 280/343
byte-identical; the 23 byte differences are the 13 in the six pre-existing groups (`dispel`,
`dispel259`, `fleeheal-408`, `knockout`, `preparation`, `vampire`), dangerfull-69 (4) and
teleotier-176 (4), plus the two `wandnames-38` VALUE lines documented above — no new unexpected
difference.
