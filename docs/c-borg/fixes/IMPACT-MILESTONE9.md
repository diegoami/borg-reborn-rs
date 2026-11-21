# Impact milestone 9: ten patch units since impact v8 (patches 0090-0099)

Review package for [milestone 9](https://github.com/diegoami/better-borg-c/milestone/9): the
verified C carries after the v8 tag `017e038e8726a7a423887a366b5d40e89c0550a1` (patches through
0089): the 0090-0096 board batch (starvealch-18, torodvalue-549, healchance-364, homebuyqty-19,
tradehist-20, torodpower-560, tporod-17) and 0097-0099 (panelrows-348 as a memory-safety carry,
runeglyph-295, necrolight-296). Runtime snapshot after the last carry's merge:
`33207fbeeb582e67d6b18e9f40613617667905ae` (PR 378). Every carry is merged, labelled
carried/open+patch-active, and receipted on its source issue. This package was added after the
snapshot in this PR — a reader checking out the snapshot will not find it there.

**Headline for reviewers:** the accumulated prior-fixture regression matrix was **rerun in full at
the v9 tip: 75 groups, 225 cases, zero unexpected mismatches** against the committed v8 baselines
(`docs/measurement/impact-v9/regression-summary.json`) — **with three newly declared intentional
transitions, all attributed to necrolight-296 (0099) by build-by-build bisection** (see below). On
the frozen v1 cohort, **all ten units were cohort-neutral**: traces byte-identical at every carry
tip; the lineage is unchanged at hp 72/83/109, depths 4/6/5, all alive at the cap. One unit
(0097) is a memory-safety carry verified with the four-phase ASan contract. Three fixed seeds are
not statistics and no survival or progress claim is made either way.

## The 10 units

| Patch | Source (issue, fix @ pristine base `e473720ed`) | Applied | Merged | Consumer | Evidence pin | Verification |
| --- | --- | --- | --- | --- | --- | --- |
| 0090 starvealch-18 | [18](https://github.com/diegoami/neo-angband-tracker/issues/18) `0fd9bc4bd` | `2260c791f` | `820bc0cbb` (PR 360) | #359 | `98aa8ad02` | `STARVEALCH-18.md` |
| 0091 torodvalue-549 | [549](https://github.com/diegoami/neo-angband-tracker/issues/549) `d4071b06e` | `187bb3b61` | `e4cbcf27b` (PR 362) | #361 | `f3323c2f0` | `TORODVALUE-549.md` |
| 0092 healchance-364 | [364](https://github.com/diegoami/neo-angband-tracker/issues/364) `b366550f7` | `73322d06c` | `f8ecd11c7` (PR 364) | #363 | `3adc1b812` | `HEALCHANCE-364.md` |
| 0093 homebuyqty-19 | [19](https://github.com/diegoami/neo-angband-tracker/issues/19) `e075f0ac7` | `d649a36fb` | `97c634a3e` (PR 366) | #365 | `40495b33a` | `HOMEBUYQTY-19.md` |
| 0094 tradehist-20 | [20](https://github.com/diegoami/neo-angband-tracker/issues/20) `9ae5e4585` | `168a43252` | `188277504` (PR 368) | #367 | `942c80a50` | `TRADEHIST-20.md` |
| 0095 torodpower-560 | [560](https://github.com/diegoami/neo-angband-tracker/issues/560) `fc444ab17` | `ce6af424b` | `71f61a615` (PR 370) | #369 | `12a59c5c4` | `TORODPOWER-560.md` |
| 0096 tporod-17 | [17](https://github.com/diegoami/neo-angband-tracker/issues/17) `d752bad27` | `3206b884b` | `9a428ea38` (PR 372) | #371 | `92ca00fdc` | `TPOROD-17.md` |
| 0097 panelrows-348 | [348](https://github.com/diegoami/neo-angband-tracker/issues/348) `82702575b` | `ee1c1bf6e` | `521c880a5` (PR 374) | #373 | `a9303c7ca` | `PANELROWS-348.md` |
| 0098 runeglyph-295 | [295](https://github.com/diegoami/neo-angband-tracker/issues/295) `50e64a37a` | `8b96bd479` | `1f0d50768` (PR 376) | #375 | `3ce928e86` | `RUNEGLYPH-295.md` |
| 0099 necrolight-296 | [296](https://github.com/diegoami/neo-angband-tracker/issues/296) `ea838de67` | `c2c215614` | `33207fbee` (PR 378) | #377 | `742e04ed8` | `NECROLIGHT-296.md` |

Every carry is a `cherry-pick -x` of one reviewed C-only source commit, one patch unit per PR,
sequential on merged main. The source base predates earlier carried fixes in `borg-store-buy.c`
(urgentshop-23, starvealch-18, homebuyqty-19, tradehist-20, tporod-17), `borg-trait.c`
(starvealch-18, torodpower-560) and `borg-caution.c`/`borg-flow-misc.c` chains; every affected
cherry-pick was a 3-way merge that preserved them (verified in the merged files, asserted by each
verifier's layered-module check).

## Verification contract (same as milestone 8, with fixture-notes)

1. Per-step before-build snapshots and native gates (965/965 + 5/5) at every carry; byte-wise
   provenance per carry (source diff applied to the before module equals the product module; the
   applied commit is the only `src`/`lib` change from the before commit). 0094 tradehist-20 is a
   two-file provenance (buy + sell modules).
2. Kits used unchanged, asserted byte-identical to their run pins; rows equal the kits' recorded
   outcomes unless disclosed below.
3. **0097 panelrows-348 memory-safety contract** (0068/0070 precedent): four cores — normal and
   ASan (`-fsanitize=address -g -DNDEBUG`, Release, exactly like the source kit) at the before
   commit and the tip. Before: the three tall-terminal modes crash with SIGSEGV in the normal
   build and report a global-buffer-overflow READ of size 8 (the `borg_grids` row pointer) under
   ASan; the three wide-terminal modes report a heap-buffer-overflow READ of size 2 (the grid
   byte); sites asserted per scan function. After: all ten modes pass report-free in both builds;
   the seven in-map controls are byte-identical across the correction in both builds.
4. **0098 runeglyph-295 disclosure:** the `floor`/`trap` control rows differ from the kit's
   recorded rows on this product — the defence chain chooses no action there **before and after**
   the fix, and the logs are byte-identical across it, proving the fix is a provable no-op for
   those modes; the divergence predates the carry and stems from carried defence-chain patches
   (candidates listed: healchance-364's `borg_heal` flow, the tele-away rod family
   0091/0095/0096; no single-patch attribution claimed). The `rune` mode carries the defect's
   direct signal (`borg_glyph_here` 0→1 with no duplicate read).
5. **0093 homebuyqty-19 disclosure:** `pair` is a second flip on this product — its outcome row
   matches the kit in both phases (`buys_home_ware0=1`), but the scenario's secondary self-check
   fails before the fix because the carried store-scan corrections in the same module alter that
   check; `roomy`/`stack` controls byte-identical.
6. **0096 tporod-17 disclosure:** the kit's `warrior-buy` informational mode diverges on this
   product — with the carried sibling rod fixes (0091 defence, 0095 power) the warrior now saves
   **and completes the purchase** (`saves=1/buys=1`, passing) where the kit's pristine base could
   not afford it; attributed to the sibling patches' recorded tree scopes.
7. Full matrix at the tip: 75 groups / 225 cases against the committed v8 baselines — zero
   unexpected mismatches (three newly declared transitions, below). Tooling 136 tests (129
   carried + 4 from PR 317 + 3 from the owner's PR 358); the milestone's own verifiers add no
   unittest cases.

## The three new matrix transitions (all necrolight-296, attributed)

The first full-matrix run at the v9 tip reported exactly two failing groups: `dispel-248/living`
and `vampire-257/near-far`+`near-near`. Both were attributed by rerunning the two groups against
**every retained carry build in order** (0091 `e4cbcf27b`, 0092 `f8ecd11c7`, 0093 `97c634a3e`,
0094 `188277504`, 0095 `71f61a615`, 0096 `9a428ea38`, 0097 `521c880a5`, 0098 `1f0d50768`): all
nine reproduce the v8 baselines; the 0099 build (`c2c215614`) flips all three cases. The
semantics are the fix working as designed:

* **vampire-257 `near-far`/`near-near`**: the scenario Necromancer stands in a lit square. The
  fix makes the Vampire Strike failure estimate match the game (33% vs the old 8%), so the borg
  refuses the spell over its 25% limit and takes no action (`acted=0 keys=0 cast_spell=0` —
  exactly the kit's recorded lit-mode after-row) where the pre-fix borg cast on the undershot
  estimate. The `far` control keeps its out-of-range refusal in both. The refusal rows are now
  pinned via `rows_must`, and the modes are declared `fail` (the fixture's own cast expectation
  was written for the buggy estimate).
* **dispel-248 `living`**: on the lit square the spell gate closes (`GATE spell_okay=0`,
  `RATING value=0` where the fixture's rating contract expects `dam + reward − mana cost`), so
  the borg no longer rates the cast; the scenario's own game-level cast still applies the damage
  and the OUTCOME rows are unchanged (`hp_before=225 hp_after=159 hp_lost=66 undead=0`,
  equal to the baseline and now pinned). Declared `fail` with the rating-contract reason.

`tools/regression-matrix.py` (otherwise unchanged since v8) records the declarations with this
attribution, and the committed summary is the declared-green rerun at the snapshot build
(`ok: true`, 225/225 cases respecting expectations).

## Frozen v1 cohort: lineage since the v8 tip

| Carry tip | Seeds 11/29/47 vs previous tip | End state |
| --- | --- | --- |
| 0090 starvealch `820bc0cbb` | all byte-identical | alive |
| 0091 torodvalue `e4cbcf27b` | all byte-identical | alive |
| 0092 healchance `f8ecd11c7` | all byte-identical | alive |
| 0093 homebuy `97c634a3e` | all byte-identical | alive |
| 0094 tradehist `188277504` | all byte-identical | alive |
| 0095 torodpower `71f61a615` | all byte-identical | alive |
| 0096 tporod `9a428ea38` | all byte-identical | alive |
| 0097 panelrows `521c880a5` | all byte-identical | alive |
| 0098 runeglyph `1f0d50768` | all byte-identical | alive |
| 0099 necrolight `33207fbee` | all byte-identical | hp 72/83/109, depths 4/6/5, alive at cap |

First fully cohort-neutral cycle: unlike milestone 8 (where spasticscore-147 diverged all three
seeds), no v9 carry changes any decision on the frozen v1 runs; the fixtures show where each
decision changes. The current diff baseline is `artifacts/nl/cohort-after` at `33207fbee`
(retained under `.worktrees/necrolight`). Harness note: the retained 0091-0098 carry builds plus
the 0099 build are what made the matrix attribution bisection above possible without any
rebuild.

## Exclusions and pending items (checked, not counted)

* **Board drained** — panelrows-348, runeglyph-295 and necrolight-296 were the last open source
  task pairs at packaging time; the remaining open source-board issues have no fix branch or
  evidence kit yet and belong to the source lane.
* **deadtake-72 recorded rows** — discharged in the milestone 8 corrections (PR 355): the lane's
  rows agree with the consumer's.
* **recallmana-157** — declared standing matrix exception, unchanged since milestone 6.

## Full-disclosure index: retained errors and corrections this cycle

* **healchance verifier (PR 364):** the kit's `scenario.c` relies on a transitive `obj-tval.h`
  include that this compiler rejects as an implicit declaration; the verifier forces the include
  rather than editing the pinned fixture.
* **torodpower PR (370):** the ledger README entry was appended twice by overlapping carry
  steps; deduplicated in a follow-up commit on the same branch (no force-push).
* **panelrows verifier (PR 374):** the ASan-before core must be built in a separate worktree
  frozen at the before commit — an in-tree rebuild after the cherry-pick silently recompiled the
  changed objects with the fixed source and produced a before-twin that already behaved
  fixed; caught because its logs matched the after rows. `OurUnitTestLib` must also be built
  explicitly in ASan trees (the default target skips it; `alltests` runs tests and fails under
  ASan).
* **runeglyph verifier (PR 376):** assertions rewritten to the observed product contract for the
  `floor`/`trap` controls with the divergence analysis above (no weakening of the defect
  requirement: the `rune` mode asserts the direct signal strictly).
* **matrix tool (this PR):** three new declared transitions with attribution. The tool is unchanged
  between the v8 runtime snapshot and the v9 runtime snapshot (verified: empty diff
  `017e038e8..33207fbee`); the only v9 change to it is the three declarations added by this PR
  (verified: `017e038e8..8a9b5307f`, 42 lines — the two `fail`/`fail_reason`/`rows_must` blocks and
  the matching docstring).

### Milestone 9 review corrections (M9-01..M9-07, applied in the corrections PR)

The milestone 9 review (issue #380) returned **approve with corrections**. All corrections are
documentation and ledger bookkeeping; none weakens a test assertion, changes a fixture, changes a
matrix declaration or alters runtime behaviour. Applied:

* **M9-01:** the 0099 merge commit was abbreviated wrongly as `33207fbeb` in four places (this
  table, the cohort table, the diff-baseline sentence and the matrix-tool sentence); the real
  commit is `33207fbee` (`33207fbeb` does not resolve). Corrected. The full SHA in the prose and
  in `docs/measurement/impact-v9/regression-summary.json` were already correct.
* **M9-02:** nine of the ten v9 verifier docstrings carried a copy-pasted fixture line naming the
  **banisharte-37** kit (`97bcf2a98074`/`bbc5de859c`) instead of their own. Their `PIN`/`EVIDENCE`
  constants were correct, so the runs are unaffected; the docstrings now name their own pins.
  This is the same defect class as M8-01 (milestone 8), applied there in PR #355.
* **M9-03:** three v9 verifier docstrings stated the opposite of their own assertions — `runeglyph`
  (the before-phase row is `read_rune=0`, not a duplicate read; the `floor`/`trap` controls fail in
  both phases), `homebuy` (`pair` fails before, it does not "pass") and `tporod` (`warrior-buy`
  passes after on this product, it does not "stay failing"). Rewritten to match the assertions and
  the corresponding carry documents; the assertions themselves were the strict and correct ones and
  are unchanged.
* **M9-04:** the ten v9 adoption issues (#359, #361, #363, #365, #367, #369, #371, #373, #375,
  #377) carried no labels, while this document claimed "labelled carried/open+patch-active". The
  `carried` and `patch-active` labels are now applied (the carries are merged, built and verified,
  which is what RULES 5 means by `carried`); the issues stay open until upstream resolves them.
* **M9-05:** the matrix-tool disclosure cited a diff range that excluded the three declarations it
  was describing; the range is corrected and the wording split so that "unchanged" is stated
  against the range that carries the change.
* **M9-06:** `patches/README.md` carried the ninety-second, ninety-fourth and ninety-sixth carry
  paragraphs twice. The ninety-sixth pair contradicted itself — the stale variant said the
  `warrior-buy` mode "stays failing by design" while the other said it "diverges on this product
  (disclosed)"; the stale variant is removed and the corrected one kept, as the ninety-fifth
  duplicate was removed in PR 370. No duplicated paragraph remains in the ledger README.
* **M9-07:** `PANELROWS-348.md` named the sanitizer error class wrongly for the tall-terminal
  modes ("heap-buffer-overflow"); the logs and the results section both say
  **global-buffer-overflow READ of size 8** (the `borg_grids` row pointer) for the tall modes and
  heap-buffer-overflow READ of size 2 (the grid byte) for the wide ones. Corrected in the
  document and in the ledger README entry.

M9-08 (milestone 9 has no issues attached, so it cannot act as a tracker) is recorded but not
applied: milestone 8 has the same empty state, so the milestone is a marker and this package is
the index, by the established convention.

## Exact next work

1. ~~PR from `task/milestone-9`~~ done: PR 379, merged `eb9a78edd`.
## Tag

Placed. `4.2.6-impact-v9` is an annotated tag on the runtime snapshot
`33207fbeeb582e67d6b18e9f40613617667905ae`, created 2026-11-09 under
explicit owner authorization, after the independent review's
approve-with-corrections verdict (#380) and the application of its
corrections (PR 381, `24eb7666e`; state recorded in PR 382). The tag
message records the snapshot, the review outcome, the matrix result and
the authorization.
