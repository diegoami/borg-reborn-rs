# Milestone 14: six carries since impact v13

Package for the milestone-14 review. It records what the cycle contains, where the evidence is,
and what is **not** measured. The review map (`docs/review/MILESTONE14.md`) holds the reviewer's
checks and prompt.

**This is not a self-review.** Everything here is the implementer's account; the review issue is
where an independent verdict goes.

## Window

| | |
| --- | --- |
| Base | tag `4.2.6-impact-v13` on runtime snapshot `9ccb2aaee` (19-carry window, milestone 13) |
| Head | `cdfb6d477` (PR 539, the last `src` change; `origin/main` is at `857307ebc`, session/docs after) |
| Size | 11 first-parent merges, 453 files changed, +96385/-703 (the bulk is committed kit evidence and the death registry) |
| Runtime surface | `src/`: 5 files, +37/-17 — one file per carried unit |
| Compare view | https://github.com/diegoami/better-borg-c/compare/4.2.6-impact-v13...cdfb6d477 |

The window also contains: the milestone-13 review record and its closure (#521: Kimi,
approve-with-corrections -> re-review **approve**, all findings reproduced), the milestone-13
corrections (PR 531: the swap/knockout fixture portability fix M13-F1 + two doc nits), the
baseline re-point (PR 532: six drift groups' matrix baselines moved to committed product logs;
the knockout `PROFILE` diagnostic prints the basename), the death registry extension (PR 533:
D0020-D0030 committed as raw, explicitly unassessed records; aggregate regenerated at 30
dossiers / 27 distinct starts / 38 attempts), and the recorder-producer removal (PR 530, owner
merged: the disposable C recorder producer extracted to the separate angband-recorder repo;
tools only, no `src/borg` code).

## The six carries

Every unit is one `cherry-pick -x` of a reviewed C-only source commit, the unchanged source kit
committed under `docs/audit/c-reports/`, a per-unit verifier (compiling each kit phase against
its own build tree), a before/after native gate (965/965 unit checks and 5/5 frontend tests at
the before commit and the unit tip, local and hosted green), byte-wise provenance, a ledger
entry, an adoption issue with a receipt, and one PR. In five of six kits **every product log is
byte-identical to the source lane's captures**; the exception is deepdescent-590, whose four
product logs differ from the pristine captures by exactly one added line — the `TRACK # Fleeing
(excessive danger)` diagnostic of the carried source156 fleeing-guard — disclosed in its unit
doc, with OUTCOME rows identical and the between-phase control identity holding everywhere.

| Patch | Source | Consumer issue | PR / merge | What it fixes | Kit result |
| --- | --- | --- | --- | --- | --- |
| 0138 RETREATADJ-154 | 154 (CBUG-0138) | 523 | 524 / `88cd97ba0` | the strategic-retreat faster-monster check never rejected a step (`break` only left the monster scan); the route is now skipped | spider fail->pass, ant control byte-identical |
| 0139 DEEPDESCENT-590 | 590 | 525 | 526 / `b033d52af` | Deep Descent (delayed, lands below deepest level) is no longer read as an instant about-to-die escape; Phase Door instead | dd-phase fail->pass, Teleport-Level control byte-identical |
| 0140 MINDBLAST-591 | 591 | 527 | 528 / `c893255d7` | Mind Blast danger scored as the 3.x-era `spell_power/2+1`; now the 4.2 8d8 maximum plus the confusion term | nar fail->pass (40->74), illusionist control byte-identical |
| 0142 RUBBLEMARK-192 | 192 | 534 | 535 / `d6fc9a68e` | the removed-rubble handler walked a 2x2 block and only tested the goal grid; it now converts the full 3x3 around the borg | dark fail->pass (0->5 converted), lit control byte-identical |
| 0143 RESIELEC-174 | 174 | 536 | 537 / `ae420e82c` | the resist-electricity acting pass logged the acid routine's note; it now names RElec (queued keys identical) | elec fail->pass, plain-melee control byte-identical |
| 0144 TPORTCHECK-593 | 593 | 538 | 539 / `cdfb6d477` | the teleport landing-zone check required both a teleport source and an escape staff while its callers ask for either; with scrolls-but-no-staff the Borg read Destruction next to the danger | scrolls fail->pass, has-both control byte-identical |

## Blocked, not carried (both with red retained and isolation evidence)

* **0141 source595 (Phase Door landing distances)** — the source fix requires exactly-`want`
  distance landings where the game lands *closest to* `want`, collapsing the candidate set on
  maps without such grids: the accumulated matrix rejected it (12 failed cases: cold-protection,
  shot-path, per-spell). Diagnosis posted on [tracker #595](https://github.com/diegoami/neo-angband-tracker/issues/595);
  red on branch `task/carry-phasemodel-595`. Re-carry when the selection is amended to the
  game's closest-to rule.
* **0145 source629 (dive gates unconditional)** — the correction passes its own kit but breaks
  the carried deskstair-153 desperation search in `twodown` (`desperate_stairs` 1 -> 0): the
  hoisted gates run ahead of the desperation bypass, which by design skips them. Isolated by
  rerunning the deskstair kit against the pre-fix build (all three modes pass there; the only
  `src` delta is the correction). Diagnosis posted on
  [tracker #629](https://github.com/diegoami/neo-angband-tracker/issues/629); red on branch
  `task/carry-divegate-629`. Re-carry when the fix preserves the desperation bypass.

## Ledger

`patches/series.json` has 142 entries, 140 `patch-active` (0001 and 0002 remain retired by the
official import). `patches/README.md` carries every window entry (Carry 137-142).

## Matrix

Fresh full matrix at the head (gate and matrix at `857307ebc`, `src/` identical to the runtime
head): **121 groups / 371 cases, `ok: true`, zero failed cases, 316/371 byte-identical**
(`docs/measurement/impact-v14/regression-summary.json`). The byte differences are the declared
intentional transitions (12: dispel-248, vampire-257, dispel259, fleeheal-408, preparation) plus
3 information-level lines from this window's own fixes: losecon-61 (2) and deepdescent-590 (1,
`control`) each gain `TRACK # No Teleport. scary squares: 100` from the 0144 gate now running in
scroll-holding scenarios — decisions unchanged, OUTCOME rows identical, both groups pass; the
same gate's either-kind behaviour is the 0144 fix working as intended beyond its kit. 40 cases
are tracker-pinned kit groups with no committed local baseline and are neither identical nor
different. All six drift groups from the milestone-13 disclosures are byte-identical after the
re-point (PR 532) and no longer appear.

**Where the summary lives:** the recorded summary is committed from the package merge onward
(PR 541); it is *not* present at the runtime snapshot `cdfb6d477`, because the file was added by
this package after the window closed. Anyone auditing at the runtime snapshot should regenerate
per the commands in the review map, or read the file at `origin/main`.

## What is not measured (and is owed)

**No per-mode arm was run.** The owner's 2026-11-10 rule requires each reachable mode
configuration (unrestricted, munchkin/Lunal only, normal only) to be tested before a per-mode
claim. All six units record their per-mode effect as **not measured** (not neutral); 0143 is nil
by construction (log-text only, queued keys identical). Reachability per the unit docs: 0138 in
the hurt-Borg retreat scan; 0139 in the about-to-die escape ladder; 0140 wherever
`borg_danger()` scores a Mind Blast caster; 0142 in the dig-message handler (light-gated);
0143 in the resist-electricity defence note; 0144 in the Destruction/Teleport-Level defence
paths. No frozen-v1 cohort run was made for this window. The batch measurement with explicit
arms belongs to `diegoami/borg-zoo` issue 87 (handoff filed). Until it runs, the claim of this
milestone is **"each carried defect is fixed and verified on its unchanged source fixture, every
native gate is green, and the accumulated matrix holds"**, not "the Borg plays better".

## Disclosures

* **Two source fixes were rejected by the accumulated matrix and not carried** (0141, 0145
  above). Both rejections were isolated against pre-fix builds before the diagnoses were posted
  to the source lane. This is the accumulated matrix doing its job: kit-level success is not
  sufficient.
* **0144's gate now runs in scroll-holding scenarios beyond its kit** (losecon-61, deepdescent-590
  each gain one `TRACK # No Teleport. scary squares: 100` line; decisions unchanged, OUTCOME rows
  identical). Documented in `docs/measurement/TPORTCHECK-593.md`; both groups' matrix baselines
  were re-pointed in this window (PR 532 pre-dates these lines, so they appear as information in
  this window's matrix and join the next re-point sweep).
* **The M13-F1 correction (PR 531) is in this window**: the swap/knockout fixtures' hardcoded
  profile paths were the one substantive milestone-13 finding (fresh-clone non-reproducibility);
  the fixtures now take the runner's per-case profile and the matrix was reproduced independently
  by the milestone-13 reviewer on a clean machine (re-review: approve).
* The knockout fixture's `PROFILE` diagnostic prints the basename (PR 532) for the same
  reproducibility reason. No decision code, input or assertion changed in either fixture.
* D0020-D0030 are committed as **raw, explicitly unassessed** collector records (PR 533); the
  per-case overall assessments remain owed (issue 163). No diagnosis or claim is made from them.
* Source-side status: every carried fix is a reviewed commit on a source branch of
  `neo-angband-tracker`; carrying is adoption, not an upstream acceptance claim. The source
  lane's own upstream filing gate is unchanged and not this lane's gate. No source-tracker or
  official posting, no borg-rs work.
