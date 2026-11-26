# Milestone 15: official master 574b618c3 and three upstream retirements

Package for the milestone-15 review and release `4.2.6-impact-v15`. It records what the window
contains, where the evidence is, and what is **not** measured. The review map
(`docs/review/MILESTONE15.md`) holds the reviewer's checks and prompt.

**This is not a self-review.** Everything here is the implementer's account; the review issue is
where an independent verdict goes.

## Window

| | |
| --- | --- |
| Base | tag `4.2.6-impact-v14` on runtime snapshot `cdfb6d477` (milestone 14) |
| Runtime snapshot | `b684b95b4` (PR 546, the official import; the last `src` change) |
| Release tag | `4.2.6-impact-v15` on this package's merge commit, whose `src/`, `lib/` and `CMakeLists.txt` are identical to `b684b95b4`; the tag sits on the package commit because `tools/make-release.py` reads `patches/series.json` and `docs/RELEASE-INSTALL.md` from the release ref (the `4.2.6-impact-v9.1` precedent) |
| Size | 8 first-parent commits/merges to the snapshot, 34 files, +9336/-252 (the bulk is the committed v14 matrix summary and upstream's SDL2/workflow files) |
| Runtime surface | `src/`: 15 files, +713/-199, all from official master; `src/borg/`: `borg-trait.c` only, +20/-11 |
| Compare view | https://github.com/diegoami/better-borg-c/compare/4.2.6-impact-v14...b684b95b4 |

No new carry, no improvement unit and no gameplay change of this repository's own lands in the
window. Its non-runtime items: the milestone-14 package and corrections (PRs 541, 543, already
recorded against milestone 14), the observer-only report/listing/frontend identity correction
(PR 545, tools only, M1-F15 of the recorder review), the official import (PR 546) and this
package.

## The official import (PR 546)

Merged angband/angband master `574b618c3`, 10 commits past the previously imported `42cd156ac`:

| Upstream | What | Here |
| --- | --- | --- |
| `95387b06f` Borg: check teleport landing zones with either kind of teleport | upstream version of carry 0144 (TPORTCHECK-593, #538) | byte-identical to the carry; merged without a tree change |
| `b5747369b` Borg: give the healing-chance adjustments the direction their comments state | upstream version of carry 0092 (HEALCHANCE-364, #363) | byte-identical to the carry; merged without a tree change |
| `007bea95c` Borg: keep the largest slay multiplier across equipment slots | upstream version of carry 0017 (CBUG-0239, #69) | the one conflict; resolved to the upstream hunk only |
| `574b618c3` trap display, `3c8ed38` SDL2 control values, `fb3c9ea` const warnings, `7e8bad3` Windows open dialog | non-Borg engine/front-end | merged as is |
| `eaaad70`, `ef363a0`, `0f6c993` | upstream CI workflows | land under `docs/upstream-reference/workflows/` (rename-detected); `.github/` unchanged |

The 0017 resolution changes the stored value relative to the carry in one case: upstream keeps
the largest multiplier per race slay across slots (`MAX`) where the carry OR-ed them (`|=`), so
two slots with the same race slay at multipliers 3 and 5 now store 5 instead of 7 (a value no
item has). Every reader of `BI_WS_*` (`borg-fight-attack.c`, `borg-fight-defend.c`,
`borg-fight-perm.c`, `borg-flow-stairs.c`, `borg-log.c`, `borg-power.c`, `borg-trait-swap.c`)
tests it only for truth, and the two forms are nonzero in exactly the same cases, so no current
decision changes. The kill-trait (`BI_WK_*`) lines are identical.

Retirement audit (`tools/verify-retire-574b618.py`, unchanged tracker fixtures, three builds
each behind its own native gate): before = the import with only the three upstream commits
reverted (scratch commit, not pushed); carry = pre-import `b0bacee06`; after = the import.

| Fixture | Defect mode: before -> carry / after | Control |
| --- | --- | --- |
| tportcheck-593 (`382acb7b9`) | scrolls: landing_ok 0, Destruction read -> landing_ok 1, none | has-both byte-identical x3 |
| healchance-364 (`f88fee1e3`) | danger/unique/risky heals 0/18/27 -> 100/32/20 | control byte-identical x3 |
| slay-239 (`25b011863`) | animal auto 17 = without -> 20 = with | nonanimal byte-identical x3 |

Carry and after logs are byte-identical in all eight modes. The provenance check asserts each
product module equals the before module plus that file's upstream diff. Consumer issues #69,
#363 and #538 are `fixed` and closed (RULES 5). Full account: `UPSTREAM.md`, "Import of
official master 574b618c3".

## Ledger

`patches/series.json` has 142 entries, **137 patch-active** (0001, 0002, 0017, 0092 and 0144
retired by official imports; patch files kept). `upstream_import.official_head` is `574b618c3`.
`patches/improvements.json` is unchanged (I-001, I-002).

## Gates and matrix

- Native gate at the import (`ec5778b92`, `src/` identical to the snapshot): **966/966** unit,
  **5/5** frontend. The unit count rises from 965 because upstream adds one check
  (`src/tests/game/status.c`). Hosted native and GitGuardian green at the PR 546 head.
- Full regression matrix at the import: **121 groups / 371 cases, `ok: true`, zero failed
  cases**, 316/331 baseline-comparable cases byte-identical
  (`docs/measurement/impact-v15/regression-summary.json`).
- **Every one of the 371 logs and outcome rows is byte-identical** to the committed v14 summary
  (`docs/measurement/impact-v14/regression-summary.json`) and to a fresh matrix run on the
  pre-import build. The 15 cases that differ from their recorded baselines (the 12 declared
  transitions and the 3 disclosed `No Teleport` TRACK lines from 0144) are the milestone-14
  set, unchanged. No fixture's behaviour changed in this window.
- Python tooling tests: 148/148.

## Release

`4.2.6-impact-v15` is the first release built on an imported official master. This package
changes `tools/make-release.py` to take the patch/source base from
`series.json` `upstream_import.official_head` (`574b618c3`) instead of the bootstrap baseline
`e473720ed`; otherwise the patch asset would carry 22 upstream commits and `apply.sh` would
point users at the 2026-10-27 checkout. `docs/RELEASE-INSTALL.md` and the README name the new
base and contents. The release verification (patch reproduces the tree on a fresh `574b618c3`
checkout, source asset builds and passes the gate, binary smoke under Xvfb, reproducible
tarballs) is recorded on the review issue and in the release notes, because the assets are built
from the commit that contains this file.

## Review and corrections

Review [#547](https://github.com/diegoami/better-borg-c/issues/547), run at the owner's request by an
independent Claude agent from a fresh clone (same model family as the implementer): **approve with
corrections**. It reproduced claims 2.1-2.7, including the three-tree retirement audit, the 371-case
matrix on the import and pre-import builds, and the patch and source assets byte for byte against the
posted checksums. Corrections (tooling and documentation only; no runtime change):

* **M15-F1 (medium):** `tools/verify-retire-574b618.py` located the import merge by a message search
  that also matched the PR 546 merge, so it aborted when run from the snapshot, the release commit or
  `main`, and it required the after gate at the import merge exactly. The merge is now pinned
  (`ec5778b92`); the after gate may sit on any descendant with the same game files, and the before
  commit's parent must have those files.
* **M15-F2 (medium):** with the patch now based on official master, `apply.sh` only warned on a wrong
  base, so applying the v15 patch to an older official checkout (`42cd156ac`, `e473720ed`) exited 0 and
  built without the fixes official Angband had taken over. `apply.sh` now refuses a checkout that does
  not contain the base unless `--force` is given; a newer descendant is accepted with a warning. This
  reaches releases after `4.2.6-impact-v15`, whose own `apply.sh` (in the tagged assets) only warns.
* **M15-F4 (info):** the window has 8 first-parent commits, not 7 (corrected above).
* M15-F3 (annotated-tag release build) was already fixed by PR 549. F5-F7: no issue.

Not verified by the reviewer: that the source-tracker issues were left untouched (no access from its
session), and the binary asset smoke (environment-dependent).

## What is not measured (and is owed)

**No per-mode arm was run.** The owner's 2026-11-10 rule requires each reachable mode
configuration (unrestricted, munchkin/Lunal only, normal only) before a per-mode claim. The
import's Borg hunks are reachable in every configuration (equipment notice, `borg_heal()`,
the Destruction/Teleport-Level defences); on every fixture and matrix case the product behaves
exactly as the v14 carries did, which is not a cohort measurement. The seven non-Borg official
commits are exercised by the native gate and the matrix only. No cohort run in this window; the
batch measurement with explicit arms belongs to `diegoami/borg-zoo` (issues 54/87). The
retirement audit ran normal builds only (the original slay verifier's ASan phases were not
rerun), and the slay fixture cannot tell `MAX` from `|=`.

The milestone claims: **"the official master is integrated, three carries are replaced by their
upstream versions with identical fixture behaviour, every gate is green and the accumulated
matrix is unchanged"**, not "the Borg plays better".

## Disclosures

* 0141 (source595) and 0145 (source629) remain blocked on source-lane corrections, as at
  milestone 14 (tracker #595, #629); nothing changed for them in this window.
* The source-tracker issues for the retired carries (239, 364, 593) belong to the source lane
  (checkpoint207) and were not edited by this lane.
* D0020-D0030 assessments remain owed (issue 163). No death analysis ran in this window.
* No official Angband posting, no borg-rs work, no source-tracker writes.
