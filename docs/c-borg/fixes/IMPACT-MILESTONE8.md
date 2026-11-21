# Impact milestone 8: twelve patch units since impact v7 (patches 0078-0089)

Review package for [milestone 8](https://github.com/diegoami/better-borg-c/milestone/8): the
verified C carries after the milestone7 runtime snapshot `3573d741e6dc3cafa4099fcef8b38fdb32952977`
(patches through 0077): board carries 0078-0081 (banisharte-37, restmult-149, shopline-125,
lightwest-40), the 0082/0083 stacked pair (modcredit-70, and neckdead-47 carried as an
owner-decided no-behaviour-change cleanup), 0084-0088 (lightbeam-88, hurtstun-62 as a second
owner-decided cleanup, the dimdoor-21/urgentshop-23 stacked pair, deadtake-72), and 0089
(spasticscore-147), which **resolves the deferral recorded in milestone 7** by carrying the fix on
a consumer-designed product-side fixture. One board pair, regensim-516, was resolved as **already
carried** (overlap of patch 0003) with no patch. Runtime snapshot after the last carry's merge:
`017e038e8726a7a423887a366b5d40e89c0550a1` (PR 343). Every carry is merged, labelled
carried/open+patch-active, and receipted on its source issue. This package was added after the
snapshot in PR 345 (head `5963d559c`) — a reader checking out the snapshot will not find it
there.

**Headline for reviewers:** the accumulated prior-fixture regression matrix was **rerun in full at
the v8 tip: 75 groups, 225 cases, zero unexpected mismatches** against the committed v7 baselines
(`docs/measurement/impact-v8/regression-summary.json`). On the frozen v1 cohort, **all three seeds
changed in this cycle** and all three are alive at the cap: the lightwest-40 carry diverged seeds
29/47, the lightbeam-88 carry diverged seed 47, and the spasticscore-147 carry diverged **all
three seeds** (final state hp 72/83/109, depths 4/6/5) — the largest cohort-wide change of any
carry so far, expected for a fix that changes where the bored search places itself on every
level. Two of the twelve units are owner-decided cleanups asserted byte-identical. Three fixed
seeds are not statistics and no survival or progress claim is made either way.

## The 12 units

| Patch | Source (issue, fix @ pristine base `e473720ed`) | Applied | Merged | Consumer | Evidence pin | Verification |
| --- | --- | --- | --- | --- | --- | --- |
| 0078 banisharte-37 | [37](https://github.com/diegoami/neo-angband-tracker/issues/37) `caa6c6acc` | `8f2669a1f1` | `4d4f2c13a` (PR 308) | #305 | `bbc5de859c` | `BANISHARTE-37.md` |
| 0079 restmult-149 | [149](https://github.com/diegoami/neo-angband-tracker/issues/149) `e66740cc4` | `72e169b1ed` | `4d4f2c13a` (PR 308) | #306 | `c6052b4a76` | `RESTMULT-149.md` |
| 0080 shopline-125 | [125](https://github.com/diegoami/neo-angband-tracker/issues/125) `9d35babe8` | `db2b5b6547` | `4d4f2c13a` (PR 308) | #307 | `9ce7a0a475` | `SHOPLINE-125.md` |
| 0081 lightwest-40 | [40](https://github.com/diegoami/neo-angband-tracker/issues/40) `6dd5874b4` | `69e020d3b5` | `71995a193` (PR 310) | #309 | `8e9e1da6a5` | `LIGHTWEST-40.md` |
| 0082 modcredit-70 | [70](https://github.com/diegoami/neo-angband-tracker/issues/70) `46c6188ec` | `ebe5504da4` | `63fbc1fb7` (PR 313) | #311 | `0af6c5af04` | `MODCREDIT-70.md` |
| 0083 neckdead-47 (cleanup) | [47](https://github.com/diegoami/neo-angband-tracker/issues/47) CBUG-0031 `e9080f35f` | `ffce4c4f40` | `63fbc1fb7` (PR 313) | #312 | `3885bfcd07` | `NECKDEAD-47.md` |
| 0084 lightbeam-88 | [88](https://github.com/diegoami/neo-angband-tracker/issues/88) CBUG-0072 `828318ab5` | `ffa3cd8bd1` | `1bd2e05a5` (PR 318) | #316 | `90e5fc16bf` | `LIGHTBEAM-88.md` |
| 0085 hurtstun-62 (cleanup) | [62](https://github.com/diegoami/neo-angband-tracker/issues/62) CBUG-0046 `f1aca2e9a` | `c55e11c92c` | `8b549c2f9` (PR 330) | #329 | `8031824656` | `HURTSTUN-62.md` |
| 0086 dimdoor-21 | [21](https://github.com/diegoami/neo-angband-tracker/issues/21) `4b7a9f414` | `bd6c7ebbed` | `8b26042e0` (PR 337) | #335 | `f8b7d662b6` | `DIMDOOR-21.md` |
| 0087 urgentshop-23 | [23](https://github.com/diegoami/neo-angband-tracker/issues/23) `09c28cd59` | `b0d8940b55` | `8b26042e0` (PR 337) | #336 | `e0ecd1ef77` | `URGENTSHOP-23.md` |
| 0088 deadtake-72 | [72](https://github.com/diegoami/neo-angband-tracker/issues/72) `9492d74ce` | `f13c96b8f5` | `aa96f142a` (PR 339) | #338 | `6332987b90` (generators) | `DEADTAKE-72.md` |
| 0089 spasticscore-147 | [147](https://github.com/diegoami/neo-angband-tracker/issues/147) `6e54def8` | `169ae7d6aa` | `017e038e8` (PR 343) | #288 | consumer fixture `3d6a0953cb` | `SPASTICSCORE-147.md` |

Every carry is a `cherry-pick -x` of one reviewed C-only source commit. 0078-0080 merged from one
stacked branch (PR 308); 0082/0083 (PR 313), 0086/0087 (PR 337) likewise — each stacked step has
its own before-build snapshot and step gate. The two cleanups (0083, 0085) are owner-decided
(2026-11-08, recorded on the source issues): removal of provably dead code, asserted
byte-identical on their kits. 0089 resolves the milestone-7 deferral: the source kit's
choreography assumes the broken #146 reverse flow and exercises nothing under this product's
carried patch 0064, so the carry uses a **consumer-designed product-side fixture**
(`docs/audit/c-reports/spasticscore-147-product/`, pinned at its own commit `3d6a0953cb`), adapted
from the kit's arena, writing the `xtra` search history the production scan itself maintains, and
disclosed as consumer-authored in the PR, ledger and receipts. The source fix and its kit remain
canonical.

## Verification contract (same as milestone 7, with two fixture-notes)

1. Per-step before-build snapshots and native gates (965/965 + 5/5) at every stacked step;
   byte-wise provenance per step (source diff applied to the before module equals the product
   module; the applied commit is the only `src`/`lib` change from the step's before commit).
2. Kits used unchanged, asserted byte-identical to their run pins; rows equal the kits' recorded
   outcomes.
3. **0088 deadtake-72 exception (disclosed):** the source lane's evidence branch held only the
   generator package at carry time; its captured before/after rows follow in a later commit and
   had not been pushed (~30 minutes). The verifier asserts the rows measured on consumer cores
   (the scenario ships its own self-check) and the run pin is the generator commit itself. The
   lane's recorded rows will be cross-checked when they land; receipts amended if they differ.
4. **0089 spasticscore-147 exception (disclosed):** consumer-designed fixture as above; the
   `mismatch` mode flips the flow goal from the already-searched grid to a fresh grid, the
   `consistent` control is byte-identical.
5. Full matrix at the tip: 75 groups / 225 cases against the committed v7 baselines — zero
   unexpected mismatches. Tooling 133 tests (129 carried from v7 plus 4 added to main by the
   parallel PR 317 shared-fixture lane; the milestone's own PRs add none).

## Frozen v1 cohort: lineage since the v7 tip

The v7-tip lineage was restarted at the dedicated intermediate cohort `3e3a9a440` after PR 302
(the owner's munchkin-by-class change, not a carry) diverged all seeds; 0078-0083 were attributed
against that intermediate cohort. From the lightwest tip onward the lineage runs unbroken at the
carry tips:

| Carry tip | Seeds 11/29/47 vs previous tip | End state (hp, depth) |
| --- | --- | --- |
| 0081 lightwest `71995a193` | 29/47 behaviour-active (west-radius grids marked) | alive |
| 0082/0083 mod-neck `63fbc1fb7` | all byte-identical (valuation + cleanup) | alive |
| 0084 lightbeam `1bd2e05a5` | 11/29 identical; **47 diverged** (hp 92→93) | alive |
| 0085 hurtstun `8b549c2f9` | all byte-identical (cleanup) | alive |
| 0086/0087 dim-urg `8b26042e0` | all byte-identical | alive |
| 0088 deadtake `aa96f142a` | all byte-identical | alive |
| 0089 spastic `017e038e8` | **all three diverged** — largest cohort change so far | hp 72/83/109, depths 4/6/5, alive at cap |

Three fixed seeds are not statistics; no survival or progress claim is made either way. The
current diff baseline is `artifacts/sp/cohort-after` at `017e038e8` (retained under
`.worktrees/spastic`).

## Exclusions and pending items (checked, not counted)

* **regensim-516** — resolved as already carried (overlap): the product contains the exact
  simulation branch via patch 0003 (`0e76ee351`, source 110, carried 2026-11-06); the source kit
  used unchanged reproduces the kit's after-rows on the consumer tip. Consumer #314 closed, no
  patch entry (source250 precedent).
* **deadtake-72 recorded rows** — pending on the source lane (see verification contract item 3);
  cross-check obligation recorded on #338 and source 72 and in the checkpoint.
* **spasticscore-147 source kit** — unchanged and canonical on the pristine base; the consumer
  fixture is additional, not a replacement.
* **recallmana-157** — declared standing matrix exception, unchanged since milestone 6.

## Full-disclosure index: retained errors and corrections this cycle

* **modcredit verifier (PR 313):** generated statring assertion keys corrected (west→statring);
  the tree-level scope check rewritten to assert the fix commit's own scope plus the documented
  stacked file (neckdead-47).
* **urgentshop verifier (PR 337):** BEFORE corrected from the branch base to the step's actual
  before commit (the dimdoor step tip); scope assert adjusted to that step — caught by the
  verifier's own gate-commit check.
* **deadtake verifier (PR 339):** run pin corrected from the evidence branch's base commit to the
  generator commit itself (the evidence branch holds a single commit).
* **spasticscore fixture (PR 343):** the first fixture draft — a granite pillar whose cardinal
  leakage created an unintended support-2 super-candidate — was redesigned in the working tree to
  direct history writes with the corner-exclusion choreography **before any commit**; the pillar
  draft was never committed and is not recoverable from the repository. A later commit
  (`d92d7c53ff`) was dropped in a worktree reset during pin repair and re-committed as
  `3d6a0953cb`; the two differ only in the added `borg.txt` profile — the `scenario.c` is
  byte-identical. No product change was involved and the before-build snapshot was unaffected.
* **matrix invocation (this package):** the runner resolves baselines relative to the checkout
  holding `tools/`; the first attempt ran inside the tip worktree and failed on the swap group's
  baseline paths. Rerun from the main checkout with `--build .worktrees/m8-tip/build/native`
  (safe: every `.c`-only carry in the cycle means the include paths match the built objects).

## Review and triage

Package issue: see the milestone-8 review issue (linked when created). Per the owner's standing
instruction, independent/external review is deferred to milestones; this package is the review
input. The review prompt is provided for the owner to launch any reviewer they choose.

## Tag

Placed. `4.2.6-impact-v8` is an annotated tag on the runtime snapshot
`017e038e8726a7a423887a366b5d40e89c0550a1`, created 2026-11-09 under
explicit owner authorization (as with `4.2.6-impact-v7` on `3573d741e`),
after the independent review's approve-with-corrections verdict and the
application of its corrections (PR 355, `7915031898`). The tag message
records the snapshot, the review outcome, the matrix result and the
authorization.
