# Milestone 11: the v11 carry cycle since impact v10

Package for [milestone 11](https://github.com/diegoami/better-borg-c/milestone/11). It records what
the cycle contains, where the evidence is, and what is **not** measured. The review map
(`docs/review/MILESTONE11.md`) holds the reviewer's checks and prompt.

**This is not a self-review.** Everything here is the implementer's account; the review issue is
where an independent verdict goes.

## Window

| | |
| --- | --- |
| Base | tag `4.2.6-impact-v10` on runtime snapshot `6944d88e8` (tag object `fa0f6639`) |
| Head | `39268ca8a` (PR 470, current `origin/main` when this package was written) |
| Size | 28 first-parent merges, 610 files changed, +77973/-171 |
| Compare view | https://github.com/diegoami/better-borg-c/compare/4.2.6-impact-v10...39268ca8a |
| Runtime surface | 18 units, 11 files, **+102/-75 lines** - the whole behavioural gameplay change |

The window opens at the v10 tag, so it also contains the milestone-10 tail (PRs 422-425: review
corrections, the tag record and the kit commits) and the v11 state/handover PRs (439, 442, 445,
456). Of those, only PR 422 touches `src/`: a comment-only edit in `borg-fight-attack.h` (+2/-1)
with no behavioural effect. The behavioural runtime surface is the carry units below.

## The 18 units

Each unit is one PR: one `cherry-pick -x` of a reviewed C-only source commit (or, for 0101, the
source commit plus its consumer correction), the source kit committed unchanged, a consumer
verifier that proves the layering byte-for-byte and reruns the kit in both phases, a native gate
at the before commit and at the tip, a ledger entry, a measurement doc and a receipt comment.

| Unit | Source (tracker) | PR / merge | What it changes |
| --- | --- | --- | --- |
| 0100 STAIRPHASE-34 | 34 CBUG-0016 | 428 / `c54393aa6` | stair escape queues `<` alone |
| 0101 LUNALTAKE-73 | 73 CBUG-0057 | 430 / `c16c56405` | skip items marked bad in the Lunal take |
| 0101a correction | 73 | 433/434 / `9ca986616` | gate 0101 on the runtime munchkin mode (supersedes `fea1e39c2`) |
| 0102 FLOWKILLHALL-98 | 98 CBUG-0082 | 436 / `281e3dd8d` | per-monster skip and hall state in `borg_flow_kill()` |
| 0103 HITCHANCE-63 | 63 CBUG-0047 | 438 / `d624c29db` | armour divided by the monster's to-hit |
| 0104 TAKESTAIR-71 | 71 CBUG-0055 | 441 / `4002d0308` | pickups measured from the nearest up staircase |
| 0105 RESTPREP-108 | 108 CBUG-0092 | 444 / `423b9fc87` | `no_rest_prep` widened so it cannot wrap |
| 0106 PREPREASON-234 | 234 | 447 / `293e153ec` | current-depth prep reason copied before later calls overwrite it |
| 0107 LOSECON-61 | 61 CBUG-0045 | 449 / `0b5889caf` | con-drain fear reads constitution |
| 0108 CRUSHV2-66 | 66 CBUG-0050 | 451 / `b663b28b0` | Crush clears the ranged danger (`v2`) |
| 0109 GRANITESIM-68 | 68 CBUG-0052 | 453 / `5f7597bb7` | ranged wall test is `== FEAT_GRANITE` |
| 0110 SCOOTIDX-100 | 100 CBUG-0084 | 455 / `4ab32de77` | Shoot'N'Scoot passes the adjacent monster |
| 0111 EMPTYSLOT-60 | 60 CBUG-0044 | 458 / `3e7732edd` | first empty slot for an empty pack |
| 0112 EATHURT-80 | 80 CBUG-0064 | 460 / `536534d2d` | cure potions only when hurt |
| 0113 RINGSLOT-86 | 86 CBUG-0070 | 463 / `4a58046f8` | left ring tries every ring |
| 0114 SWAPTRIAL-31 | 31 CBUG-0012 | 465 / `b77f5ae00` | restore the pack entry after home swap trials |
| 0115 HOMESKIP-29 | 29 CBUG-0010 | 467 / `2d0840571` | reset the home-grab skip flag per ware |
| 0116 SELLCHARGES-132 | 132 CBUG-0116 | 469 / `858774476` | restore staff/wand charges after home-sell trials |
| 0117 EGOSELL-133 | 133 CBUG-0117 | 470 / `39268ca8a` | sell guard reads the sold item's ego |

18 units, 19 patch files (0101a is a correction of 0101, not a new ordinal). Ledger:
`patches/series.json` has 116 entries, `patches/` 117 files. Every unit is `patch-active` and its
consumer issue stays open.

## Evidence

* Per unit: `docs/measurement/<UNIT>.md` and `tools/verify-<unit>.py`; the kit under
  `docs/audit/c-reports/<kit>/` with the source receipt and the consumer before/after logs.
* `patches/series.json` - one entry per unit (source head, applied commits, patch, evidence pin,
  verification doc, status).
* Matrix: `docs/measurement/impact-v11/regression-summary.json` (95 groups / 290 cases at
  `39268ca8a`).
* Adoption issues: one per unit, linked from the ledger and the docs; each carries the claim and
  the completion receipt.

## Matrix

At the head: **95 groups / 290 cases, `ok: true`, no failed group**, 237/250 compared cases
byte-identical. The 13 byte differences are in six groups that predate this cycle
(`dispel`, `dispel259`, `fleeheal-408`, `knockout`, `vampire`) plus `preparation`, whose
`speed-wins`/`speed-only` logs now print `no_rest_prep=50000` where the baseline holds `-15536` -
that is 0105's wrap fix visible in an independent fixture, with identical decisions. No new
difference was introduced by the 18 units.

## What is not measured (and is owed)

**No per-mode arm was run in this cycle.** Each unit's per-mode effect is recorded as **not
measured**, not neutral: 0100-0117 are unconditional changes in shared code, and the owner's
2026-11-10 rule requires each reachable mode configuration to be tested before a per-mode claim.
The single exception is 0101a, which has a runtime-mode gate proven by a two-configuration fixture
(Lunal path vs munchkin mode); its first attempt gated on a class flag
(`borg_cfg[BORG_MUNCHKIN_START]`) and was inert for Warrior/Druid/Ranger everywhere, which is
why the rule exists.

Two single-configuration (unrestricted/default, all modes) frozen-v1 cohort runs do exist. They
are evidence for the unit, not per-mode attribution:

* **0100 STAIRPHASE-34** - seeds 11/29/47, 20k turns: alive at cap, same hp and depth, traces
  **byte-identical** before and after (issue #427). Neutral in the unrestricted configuration.
* **0101 LUNALTAKE-73** - the same cohort moved: all three traces diverge and the runs finish
  **2-3 levels shallower** (4/6/5 -> 1/4/2), survival unchanged. That regression is what produced
  the 0101a gate and is disclosed below.

Neither run separates munchkin/Lunal from normal play, and the Warrior cohort cannot enter the
Lunal path at all (it needs `BORG_SELF_LUNAL` and depth + 15). The honest summary is therefore:
one unconditional unit is cohort-neutral in the default configuration, one ungated carry was not,
and every other unit's effect in each mode configuration remains unmeasured.

The batch measurement with explicit arms belongs to `diegoami/borg-zoo` issue 54 (separate
repository and session): a mode profile that inherits borg defaults, a munchkin/Lunal arm, and a
normal arm, run against a pinned specimen. Until that runs, the claim of this milestone is
**"each defect is fixed and the fixed branch executes in the fixture, with every native gate
green"** - not "the Borg plays better".

## Disclosures

* **0101 needed a correction.** The unrestricted skip removed the items munchkin mode collects to
  sell; the take flow returned false more often and the frozen cohort oscillated climb/dive at
  shallow depth. Ref 0101a restricts the skip to outside the runtime munchkin mode.
* **0105 is a struct field width** (`int16_t` -> `int`): the fixture compiles against each phase's
  own headers, and the independent `preparation` fixture shows the timer value change.
* **0114's before-phase count differs from the source receipt** (`1/0` here vs `2/1`): the product
  tree's swap valuation differs from the patch-free source base, so the wrong path is shorter.
  The kit's pass criterion still flips and the after rows are identical.
* **Source-side state:** the source fixes are released as reviewed commits on source PRs; several
  of those PRs are still open on `neo-angband-tracker` (status `in-review`). Carrying them here is
  adoption of the released fix, not an upstream acceptance claim.
* **No upstream or source-tracker posting** was made in this cycle, and there is no Rust/borg-rs
  work.

## Milestone-review corrections (2026-11-10)

The independent review on issue 471 returned **approve with corrections** (findings F1-F5). All
five are applied in this package; none changes `src/`.

* **F1, patch artifact.** `patches/0101a-lunaltake-mode-gate.patch` had been generated against the
  superseded `fea1e39c2` blob (config-flag gate) and did not apply after 0101. It is regenerated
  against the 0101 state; replaying 0100-0117 from `4.2.6-impact-v10` now applies every patch and
  reproduces the head `src/` byte for byte.
* **F2, cohort wording.** The blanket "no cohort measurement" sentence is replaced by the two
  single-configuration runs above (0100 neutral, 0101 divergent).
* **F3, runtime surface.** The window now names the comment-only `src/` edit in PR 422.
* **F4, 0100 attribution.** STAIRPHASE-34 now carries its mode-attribution section and ledger note.
* **F5, kit provenance.** The review map's blanket rule now names the one adapted kit
  (`lunaltake-73/scenario.c`).

The review's two observations outside the window are recorded in `session.json`: official
`angband/angband` master has advanced to `42cd156ac9f9` (Borg commits among them), and the trial
merge conflicts in `borg-item-wear.c` exactly as it already does at the v10 baseline, so it is
pre-existing integration debt for a separate upstream-integration PR, not a v11 regression.

## Tag placement

`4.2.6-impact-v11` is an annotated tag on runtime snapshot `39268ca8a`, placed 2026-11-10 under
explicit owner authorization after the review's verdict (approve with corrections; F1-F5 applied
in `981be4a1d`, docs and the `0101a` patch only). The annotation records the 18 units, the review,
the matrix result and the cohort statement; the tag object is
`443ba5c485bd1fa5267a677e44370d8ddb753c15`. The corrections commit and this tag record follow on
main but are not inside the tag, exactly as the milestone 9 and 10 packages followed their
snapshots; neither changes code or a baseline row.
