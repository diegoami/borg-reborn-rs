# Milestone 13: nineteen carries since impact v12

Package for the milestone-13 review. It records what the cycle contains, where the evidence is,
and what is **not** measured. The review map (`docs/review/MILESTONE13.md`) holds the reviewer's
checks and prompt.

**This is not a self-review.** Everything here is the implementer's account; the review issue is
where an independent verdict goes.

## Window

| | |
| --- | --- |
| Base | tag `4.2.6-impact-v12` on runtime snapshot `4e1b912cb` (the official import, PR 478; tag placed by the owner 2026-11-13 after the milestone-12 review was waived, issue 479) |
| Head | `9ccb2aaee` (PR 519, the runtime head when this package was written) |
| After the head | `1f0b04882`, a session-state/docs commit; `src/` is identical to the head |
| Size | 20 first-parent merges, 501 files changed, +90086/-125 (the bulk is committed kit evidence) |
| Runtime surface | `src/`: 16 files, +155/-124 across the 19 carry units |
| Compare view | https://github.com/diegoami/better-borg-c/compare/4.2.6-impact-v12...9ccb2aaee |

The window also contains the milestone-12 package/review record (PR 480, docs only) and two
session-state commits. None of those touch `src/` beyond `session.json`.

## The nineteen carries

Twelve from the completed issue-426 queue (0119-0130, merged by the earlier leg of the batch) and
seven from the owner-directed 2026-11-13/14 source-board pairs (0131-0137). Every unit is one
`cherry-pick -x` of a reviewed C-only source commit, the unchanged source kit committed under
`docs/audit/c-reports/`, a per-unit verifier, a before/after native gate (965/965 unit checks and
5/5 frontend tests at the before commit and the tip, local and hosted green), a ledger entry, an
adoption issue with a receipt, and one PR.

| Patch | Source | Consumer issue | PR / merge | What it fixes | Kit result |
| --- | --- | --- | --- | --- | --- |
| 0119 GLYPHTRACK-116 | 116 (CBUG-0100) | 482 | 483 / `e2f47c0b8` | the Sea-of-Runes read is reported on a grid that is already tracked | fail-before/pass-after |
| 0120 LUNALTELE-193 | 193 | 484 | 485 / `81fcbce93` | Lunal mode is left when a monster teleports the Borg | fail-before/pass-after |
| 0121 DEADUNIQ-101 | 101 (CBUG-0085) | 486 | 487 / `4150d80f6` | the dead-unique table is seeded with uniques at start-up | fail-before/pass-after |
| 0122 UNIQUEDEAD-94 | 94 (CBUG-0078) | 488 | 489 / `0e7d3d569` | a killed unique's seen race is marked dead, not the race numbered like the record | fail-before/pass-after |
| 0123 REBIRTHKEY-52 | 52 (CBUG-0036) | 490 | 491 / `d1f30ffac` | respawn types the birth menu's own letters for race/class 7+ | fail-before/pass-after |
| 0124 DANGERFULL-69 | 69 | 492 | 493 / `5aaace27e` | comment: `borg_danger()` always rates the full damage on purpose | every mode passes both phases |
| 0125 VENOMCOST-111 | 111 | 494 | 495 / `a5d9aa099` | dead mana-cost test removed from the Venom preparation | every mode passes both phases |
| 0126 GLYPHTERM-115 | 115 | 496 | 497 / `437ba8ab9` | always-zero glyph count dropped from the outlying rune score | every mode passes both phases |
| 0127 TRAPDEAD-122 | 122 | 498 | 499 / `80b9227ac` | unreachable visible-trap case removed from dark-flow exploration | every mode passes both phases |
| 0128 DOORCAP-126 | 126 | 500 | 501 / `eb030ea77` | unreachable door-count test removed from `borg_play_step()` | every mode passes both phases |
| 0129 TELEOTIER-176 | 176 | 502 | 503 / `b15911c95` | unreachable fail-rate tier removed from the Teleport Other defence | every mode passes both phases |
| 0130 RESTCOMMENT-486 | 486 | 504 | 505 / `bdb373195` | comment: why `borg_check_rest()` rates a monster at its own grid | every mode passes both phases |
| 0131 WANDNAMES-38 | 38 (CBUG-0022) | 506 | 507 / `58c25689d` | the Confuse Monster and Lightning Balls wands are looked up by their own names | 4 modes fail→pass, control byte-identical |
| 0132 SWAPFEAR-202 | 202 | 508 | 509 / `795646781` | fear protection is recorded on swap weapons and armour | fail-before/pass-after, controls byte-identical |
| 0133 HEALAMOUNT-592 | 592 | 510 | 511 / `8342bafe6` | fixed 20/40/60 heal amounts replace the 3.x percentages | window fail→pass, control byte-identical |
| 0134 CURELIST-596 | 596 | 512 | 513 / `b9958bcde` | cures that do not cure the condition are not tried | blind fail→pass, control byte-identical |
| 0135 STUNPROT-589 | 589 | 514 | 515 / `79b358acb` | protection from stun is modelled separately from sound resistance (3 files, +`BI_PSTUN`) | rsound/pstun fail→pass, controls byte-identical |
| 0136 VULNRES-594 | 594 | 516 | 517 / `87f540bc9` | an elemental vulnerability cancels the matching permanent resistance | vuln fail→pass, control byte-identical |
| 0137 VULNMOD-610 | 610 | 518 | 519 / `9ccb2aaee` | unresisted elemental vulnerability ×4/3 and its temp-resistance cancellation are modelled in `borg_danger()` (**stacked on 0136**) | vuln/temp fail→pass, control byte-identical |

Byte-identical means every product log equals the source lane's captured log for that phase and
mode. The exceptions are recorded, attributed, and listed under Disclosures.

## Not carried

* **source616** was verified this cycle to be a byte-identical duplicate of carried patch 0004
  (source 26, CBUG-0004): the same one-line hunk (`res_acid` → `res_cold` in the `MONBLOW_COLD`
  case, same blob pair `11efe9f1a..15eb2732d`). Recorded on pair PR #622; not carried.
* Sources 13 and 249 still have no released source fix (249 additionally has the recorded
  masking/blocked consumer attempt on issue #140). 250 was closed upstream as a duplicate of the
  carried #67.
* The five behaviour-flavored sources 154, 590, 591, 595 (classified by the consumer lane as
  game-model bug fixes) and 615 (a policy choice: down-stairs-vs-Recall retreat, flagged for
  owner review by the source lane) are **not in this window** and await the owner's direction.

## Ledger

`patches/series.json` has 136 entries, 134 `patch-active` (0001 and 0002 remain retired by the
official import). `patches/README.md` carries all nineteen window entries (the
hundred-and-nineteenth through Carry 136); every entry links its consumer issue,
source issue, source head, applied commits, patch file, evidence ref and verification doc.

## Matrix

Fresh full matrix at the head (src identical to `9ccb2aaee`, build/gate recorded at `1f0b04882`):
**115 groups / 359 cases, `ok: true`, zero failed groups, 295/359 byte-identical.** The 24 byte
differences are the same known ones throughout the window: 13 in the six pre-existing groups
(`dispel`, `dispel259`, `fleeheal-408`, `knockout`, `preparation`, `vampire`), dangerfull-69 (4),
teleotier-176 (4), wandnames-38 (2), healamount-592 (1). Summary:
`docs/measurement/impact-v13/regression-summary.json`.

## What is not measured (and is owed)

**No per-mode arm was run.** The owner's 2026-11-10 rule requires each reachable mode
configuration (unrestricted, munchkin/Lunal only, normal only) to be tested before a per-mode
claim. All 19 units record their per-mode effect as **not measured** (not neutral):

* The seven 0119-0123 behaviour fixes and the seven 0131-0137 fixes are unconditional shared code;
  the unit docs state which configurations the changed branches reach (0120 only with Lunal mode
  on; 0121/0122 only with a dead unique or a shape-changed killed unique at start-up; 0123 only on
  respawn; 0131 at every start-up; 0132 on every gear notice; 0133/0134 wherever `borg_heal()` /
  `borg_recover()` run; 0135-0137 in every `borg_danger()` call).
* The seven 0124-0130 comment/removal cleanups are behaviour-preserving by construction
  (comments only, or code shown unreachable); the kits show every mode passing in both phases
  with identical outcomes.
* No frozen-v1 cohort run was made for this window.

The batch measurement with explicit arms belongs to `diegoami/borg-zoo` issue 54 (separate repo
and session). Until it runs, the claim of this milestone is **"each carried defect is fixed and
verified on its unchanged source fixture, every native gate is green, and the accumulated matrix
holds"**, not "the Borg plays better".

## Disclosures

* **Capture drift (4 groups, information not failure).** dangerfull-69 and teleotier-176 (4
  cases each) and wandnames-38 (2) and healamount-592 (1) carry product logs that differ from the
  source lane's pristine captures in one diagnostic line while their OUTCOME/STATE rows and
  before/after product logs match exactly: `grid_danger`-style values changed by earlier carries
  (dangerfull-69, teleotier-176), the confusion/fear bolt VALUE −6 → −5 on the carried layer
  (wandnames-38), and the flee reason `Fleeing (failed to teleport)` → `Fleeing (excessive
  danger)` from the carried source156 fleeing-guard (healamount-592). All four groups join the
  owed follow-up of re-pointing their matrix baselines from the pristine captures to the product
  logs (recorded on checkpoint 1). No behaviour claim rests on those lines.
* **Retained verifier correction `37ded9e8c` (0135).** The first 589 verifier draft compiled both
  kit phases against the working tree's headers; the fix inserts `BI_PSTUN` into the
  `borg-trait.h` enum, so the before phase read every later trait through shifted indices (isstun
  read 0 and the calm run cleared the wrong slots, making the pstun before-row appear to pass).
  The pristine `42cd156ac` build, the v11 build and every carried build were bisected with the
  same cross-compiled scenario before the header cause was found. Each phase is now compiled
  against its own build tree (`CMAKE_HOME_DIRECTORY`), and 0137 uses the same pattern. No product
  assertion changed.
* **Stacked provenance (0137).** The 610 source fix is stacked on the 594 source fix. The
  verifier's provenance base is therefore the stacked source parent `82fd25a54` (content provided
  by the consumer's 0136), and the kit's before-tree already contains the 594 correction.
* **616 duplicate.** The duplicate verification (same hunk, same blob pair as carried 0004) was
  done by diffing the source commit against `patches/0004-cold-melee-protection.patch`; no kit was
  run for it, and none is needed.
* Source-side status: every carried fix is a reviewed commit on a source branch of
  `neo-angband-tracker`; carrying is adoption, not an upstream acceptance claim. The source lane's
  own upstream filing gate (official PRs #6817/#6818) is unchanged and not this lane's gate. No
  source-tracker or official posting, no borg-rs work.
