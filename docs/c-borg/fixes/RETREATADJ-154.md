# source154: skip a strategic-retreat step next to a faster monster (patch 0138)

Consumer adoption of [source 154](https://github.com/diegoami/neo-angband-tracker/issues/154)
(CBUG-0138), source pair [#587](https://github.com/diegoami/neo-angband-tracker/issues/587),
fix `3b0a94673` (branch `task/retreatadj-154`), kit `docs/audit/c-reports/retreatadj-154` from the
evidence branch `task/retreatadj-154-evidence` (generators byte-identical to the fixtures run-state
pin `f87cada632`; the captured artifacts were added by later evidence-branch commits; the
interpreter `__pycache__` directory is not carried).

## Change

When a monster makes its grid too dangerous, the hurt Borg looks for a strategic retreat: a safer
grid in view and a short path to it. For the first step of each candidate route it means to skip a
grid next to a monster faster than itself ("He will just hit me when I land on that grid"). The
check found such a monster but its `break` only left the monster scan, so the test had no effect
and the step was taken anyway. Record the match and skip that route.

Source `3b0a94673` -> applied `634ac64fc` (`cherry-pick -x`), patch `patches/0138-retreat-adjacent-skip.patch`,
scope `src/borg/borg-caution.c` only (7 insertions, 1 deletion). The module carries the I-001/I-002
improvements and the source364/back-away/desperate-stairs/recall-mana/fleeing-guard/healamount
corrections; the source hunks apply at a +36-line offset and the provenance check below is
byte-wise over the layered result.

## Verification

Before commit `3da903a11` (the kit commit, `src/` identical to `origin/main`), tip `fb31c5fed`, kit
unchanged (`tools/verify-retreatadj154.py`). Native gates green at both (965/965 unit checks and
5/5 frontend tests at the before commit and at the tip). Byte-wise provenance: the source diff
applied to the before module equals the product module, and the `src`/`lib`/build scope of the fix
and of before..tip is the one file. The verifier compiles each kit phase against that phase's own
build tree (the per-phase include-roots pattern from the source589 verifier).

| Mode | Before | After |
| --- | --- | --- |
| `spider` (faster cave spider diagonally beside the first step) | fails: the Borg retreats into the grid next to it ("Retreating to 12,15 … via 12,15 (24 > 8)", `retreat=1`, `first_step_next_to_side=1`) | passes: the route is skipped, no retreat this turn (`retreat=0`, `first_step_next_to_side=0`) |
| `ant` (soldier ant, not faster — control) | passes: the same retreat proceeds | passes, same outcome, **byte-identical** |

In both modes the step's own danger (8 with the spider, 2 with the ant) is below the first-step
limits (40 % of hit points, half the current danger), so only the faster-monster check can reject
the route — the defect is not masked here.

The committed `before-*.log` / `after-*.log` are the source lane's captures. All four product
logs are **byte-identical** to them (both phases, both modes) and the STATE/OUTCOME rows match
exactly; the per-case comparison is recorded in `product-result.json`, and the source's own rows
and hashes are in `receipt.json`, carried unchanged.

## Mode attribution

Per the 2026-11-10 owner rule, each configuration is stated. The strategic-retreat scan runs in
`borg_caution()` in every configuration — unrestricted (all modes), munchkin/Lunal-only and
normal-only — whenever the hurt Borg looks for a retreat, so the changed branch executes in all of
them; the kit shows it executing (the spider mode's route is rejected after the fix; the ant
control's route survives because its monster is not faster). The per-mode cohort effect of not
retreating into a faster monster's attack range is **not measured**, not neutral; the batch
measurement with explicit arms is owed to borg-zoo issue 87. Not measured, per the source: what
the Borg does instead on following turns (here `borg_caution()` takes no action this turn), the
fight, deaths, or how often a weak faster monster stands beside the only retreat route.

Matrix group `retreatadj-154` (2 cases) uses the `after-*.log` rows as its baseline and is
byte-identical in both cases. Full matrix at this tree: 116 groups / 361 cases, `ok: true`, zero
failed cases, 297/361 byte-identical; the 24 byte differences are the same known ones as before
this carry — no new difference.
