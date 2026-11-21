# Milestone 12: one carry and the official import since impact v11

Package for the milestone-12 review. It records what the cycle contains, where the evidence is,
and what is **not** measured. The review map (`docs/review/MILESTONE12.md`) holds the reviewer's
checks and prompt.

**This is not a self-review.** Everything here is the implementer's account; the review issue is
where an independent verdict goes.

## Window

| | |
| --- | --- |
| Base | tag `4.2.6-impact-v11` on runtime snapshot `39268ca8a` |
| Head | `4e1b912cb` (PR 478, `origin/main` when this package was written) |
| Size | 6 first-parent merges, 64 files changed, +9790/-164 |
| Compare view | https://github.com/diegoami/better-borg-c/compare/4.2.6-impact-v11...4e1b912cb |
| Runtime surface | 32 files, +674/-128: one carry (`borg-flow-kill.c`, -18) and the official import |

The window also contains the milestone-11 package, its review corrections and the tag record
(PRs 472-474) and the v11 state PR (477). None of those touch `src/`.

## The two changes

| Change | Source | PR / merge | What it does |
| --- | --- | --- | --- |
| 0118 NEWKILL-95 | tracker 95 (CBUG-0079) | 476 / `a39d23d8f` | `borg_new_kill()` no longer forces a newly seen wall-walker's grid to floor (the race was read from the slot before the new race was stored) |
| Official import | `angband/angband` `42cd156ac` | 478 / `4e1b912cb` | merges 12 official commits past `e473720ed` (5 Borg, 7 non-Borg); retires carries 0001 and 0002 |

### 0118 NEWKILL-95

One `cherry-pick -x` of the reviewed C-only source commit `1a220cb2d`, the unchanged kit at
`docs/audit/c-reports/newkill-95/`, `tools/verify-newkill95.py`, `docs/measurement/NEWKILL-95.md`.
On the unchanged fixture the wall mode keeps the granite under the wall-walker and queues no shot;
the floor control is byte-identical. The code is shared sighting logic with no mode gate.

### Official import (PR 478)

* One merge conflict, in `borg_backup_swap`, resolved to the upstream form. The other local carries
  in `borg-item-wear.c` are untouched. Upstream moved `borg_dont_react` into `borg.dont_react`, so
  `generate_keypress` in `borg.c` reads the field again.
* **Retired carries:** 0001 (CBUG-0066 safer swap, upstream `bc67c099e`) and 0002 (CBUG-0002
  knockout gate, upstream `9d062a64d`, `8c75bdccb`, `42cd156ac`). `patch_active` is false in
  `patches/series.json`; the patch files are kept.
* **Evidence:** unchanged tracker fixtures `1b67370d` (swap: 5 fail-before/pass-after, 4 controls
  byte-identical, 79 -> 26 HP and 113 -> 59 HP) and `3de1e312` (knockout: kill reactions 0 -> 1,
  suppression 1 -> 0, 3 controls byte-identical). "Before" is the import head with the five upstream
  Borg commits reverted (recipe in `UPSTREAM.md`). The knockout fixture reads the removed global, so
  `tools/verify-knockout.py` and `tools/regression-matrix.py` map it at compile time; the fixture file
  is unchanged.
* An independent PR review (a review on PR 478) returned request-changes on one tooling
  blocker (the regression-matrix knockout group did not compile) plus accuracy/ledger gaps M2-M6; all
  were applied in `8de6ed459` before merge. M7 (close consumer issues 4 and 7 as `fixed`) is done.
* Native gate on the merge and the hosted `native` check on the merge head: green
  (965/965 unit, 5/5 frontend).

Ledger: `patches/series.json` has 117 entries, 115 `patch-active`.

## What is not measured (and is owed)

**No per-mode arm was run.** The owner's 2026-11-10 rule requires each reachable mode configuration
(unrestricted, munchkin/Lunal only, normal only) to be tested before a per-mode claim.

* 0118 is shared sighting code; its per-mode effect is **not measured**.
* The import's merged hunks (upstream's `b_p1`/`b_p2 = 9999` initialisation and removed early return in
  `borg_backup_swap`, the single-gate knockout reaction) are **not measured** per mode. The fixtures
  show equivalence on constructed scenarios only.
* The 7 non-Borg official commits (rest handling, `TERM_XTRA_CLEAR`, object-file ordering, Windows
  open dialog) are exercised only by the native gate.
* No frozen-v1 cohort run was made for this window.

The batch measurement with explicit arms belongs to `diegoami/borg-zoo` issue 54 (separate repo and
session). Until it runs, the claim of this milestone is **"the defect is fixed, the retired carries
are covered by upstream, and every native gate is green"**, not "the Borg plays better".

## Disclosures

* A build from pristine `e473720ed` objects cannot run against the new `lib/gamedata`
  (`player_timed.txt` flag format), so the retirement "before" tree is the merged tree with upstream's
  Borg hunks reversed, not a pristine build.
* The comparison against pristine is therefore of the Borg hunks only; the 7 non-Borg commits are in
  both phases.
* The tracker fixture commits are not in this repository; they were fetched by SHA from
  `neo-angband-tracker`.
* Source-side status: the source fix for 0118 is a reviewed commit on a source PR; carrying it here is
  adoption, not an upstream acceptance claim. No source-tracker or official posting, no borg-rs work.
