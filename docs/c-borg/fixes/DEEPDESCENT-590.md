# source590: do not use Deep Descent as an emergency escape (patch 0139)

Consumer adoption of [source 590](https://github.com/diegoami/neo-angband-tracker/issues/590),
source pair [#604](https://github.com/diegoami/neo-angband-tracker/issues/604),
fix `e8c8cae0d` (branch `task/deepdescent-590`), kit `docs/audit/c-reports/deepdescent-590` from the
evidence branch `task/deepdescent-590-evidence` (generators byte-identical to the fixtures run-state
pin `860efeb900`; the captured artifacts were added by later evidence-branch commits; the
interpreter `__pycache__` directory is not carried).

## Change

Deep Descent is used as an instant emergency escape, but it is delayed (it takes effect on a later
turn, after the round of danger that prompted it) and lands below the deepest level reached — the
opposite of an escape from an about-to-die position. In the about-to-die escape ladder the Borg
read a Scroll of Deep Descent as the escape. Read Phase Door (or the other instant escapes)
instead in that ladder; Deep Descent stays available where it is not an escape.

Source `e8c8cae0d` -> applied `292534621` (`cherry-pick -x`), patch `patches/0139-deep-descent-escape.patch`,
scope `src/borg/borg-escape.c` only (14 insertions, 7 deletions). The module carries earlier
consumer corrections (recall-depth answer, escape-choice work); the source hunks apply at
+11..16-line offsets and the provenance check below is byte-wise over the layered result.

## Verification

Before commit `5088aaea8` (the kit commit, `src/` identical to `origin/main` at `88cd97ba0`,
which includes the carried 0138), tip `0b9b190ba`, kit unchanged
(`tools/verify-deepdescent590.py`). Native gates green at both (965/965 unit checks and 5/5
frontend tests at the before commit and at the tip). Byte-wise provenance: the source diff applied
to the before module equals the product module, and the `src`/`lib`/build scope of the fix and of
before..tip is the one file. The verifier compiles each kit phase against that phase's own build
tree (the per-phase include-roots pattern from the source589 verifier).

| Mode | Before | After |
| --- | --- | --- |
| `dd-phase` (about-to-die band, 44/119 HP, danger 89, Deep Descent carried) | fails: the Borg reads the Deep Descent scroll (`reads_deep_descent=1`) | passes: Phase Door read instead (`reads_phase=1`) |
| `control` (Teleport Level — an instant escape) | passes: Teleport Level read | passes, same outcome, **byte-identical** |

The committed `before-*.log` / `after-*.log` are the source lane's captures. STATE/OUTCOME rows
match them exactly in every mode and phase, and the before/after product logs match each other
(the control is byte-identical between the phases). Every product log differs from the pristine
capture by exactly one added diagnostic line, `TRACK # Fleeing (excessive danger)`: the carried
source156 fleeing-guard reports the flee reason for this position, which the pristine base does
not emit. Carried-layer difference, decisions identical; the group joins the owed baseline
re-point follow-up on checkpoint 1 (the dangerfull-69/healamount-592 pattern). The per-case
comparison is recorded in `product-result.json`, and the source's own rows and hashes are in
`receipt.json`, carried unchanged.

## Mode attribution

Per the 2026-11-10 owner rule, each configuration is stated. The about-to-die escape ladder runs
in the escape path in every configuration — unrestricted (all modes), munchkin/Lunal-only and
normal-only — so the changed branch executes in all of them; the kit shows it executing (the
dd-phase mode flips the scroll read to Phase Door) and records the control where the corrected
ladder is unchanged (Teleport Level, an instant escape, keeps its place). The per-mode cohort
effect of not "escaping" into a delayed scroll is **not measured**, not neutral; the batch
measurement with explicit arms is owed to borg-zoo issue 87.

## Disclosures

* A first attempt at this unit was staged on a branch rebased from a stale local `main` ref
  (`1f0b04882`), which lacked the merged 0138; the mistake was caught before any PR (the branch
  would have reverted the 154 fix in `borg-caution.c`). The branch was rebased onto `88cd97ba0`,
  the verifier constants were updated, and both gate builds, the verifier and the matrix were
  re-run from scratch on the correct base; the earlier runs are discarded and not part of this
  record.

Matrix group `deepdescent-590` (2 cases) uses the `after-*.log` rows as its baseline; the added
`TRACK` line above is recorded information, not a failure. Full matrix at this tree: 117 groups /
363 cases, `ok: true`, zero failed cases, 297/363 byte-identical; the 26 byte differences are the
24 known ones (13 in the six pre-existing groups, dangerfull-69, teleotier-176, wandnames-38,
healamount-592) plus the two `deepdescent-590` TRACK lines documented above — no new unexpected
difference.
