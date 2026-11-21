# Impact milestone 7: thirteen patch units since impact v6 (patches 0064-0077)

Review package for [milestone 7](https://github.com/diegoami/better-borg-c/milestone/7): the
verified C carries after the milestone6 runtime snapshot `b3c825c97c8aef18b252a7fb7d9b7dceef83d586`
(63 carries): patches 0064-0067 (reverse-flow pair, swap acid element, munchkin stair-cycle),
owner-decided corrections 0068/0069 (source287 buffer size on 0118, source213 `square_isglow` on
0050), and board carries 0070-0077 (iswarded-198 memory safety, painmsg-197, trollswap-201,
drophole-136, wearskip-85, lantern-90, homebanish-54). Spasticscore-147 was prepared as 0072 and
**deferred before its PR** (see exclusions). Runtime snapshot after the last carry's merge:
`3573d741e6dc3cafa4099fcef8b38fdb32952977` (PR 299); docs-only commits after the snapshot carry
this package and the regression-matrix tool. Every carry is merged, labelled
carried/open+patch-active, and receipted on its source issue.

**Headline for reviewers:** the matrix gap from milestone 6 is closed — the accumulated
prior-fixture regression matrix is committed as a tool (`tools/regression-matrix.py`) and was
**rerun in full at the v7 tip: 75 groups, 225 cases, zero unexpected mismatches** (see
verification contract item 5 and `docs/measurement/impact-v7/regression-summary.json`). On the
frozen v1 cohort, seed 47 — dead to a
cave spider at the v6 tip — is **alive at the cap in the v7 lineage** (all three seeds alive:
hp 28/49/58); the pair's own cohort run shows seed 47 alive immediately after the
0064/0065 reverse-flow pair (measured as a unit; no attribution to one patch within the
pair), and
0071 (pain messages) is the milestone's one other behavior-active carry (seeds 11/29 diverge,
alive at cap). Three fixed seeds are not statistics and no survival or progress claim is made
either way.

## The 13 units

| Patch | Source (issue, fix @ pristine base `e473720ed`) | Applied | Merged | Consumer | Evidence pin | Verification |
| --- | --- | --- | --- | --- | --- | --- |
| 0064 spastic-revflow | [146](https://github.com/diegoami/neo-angband-tracker/issues/146) `f97311e5d` | `dafc4bd45` | `8d0d0cd8b` (PR 276) | #268 | `da9a39f4da` | `REVERSE-FLOW.md` |
| 0065 glyph-revflow | [345](https://github.com/diegoami/neo-angband-tracker/issues/345) `f61838129` | `e66b1dca2` | `8d0d0cd8b` (PR 276) | #269 | `fbe06401c5` | `REVERSE-FLOW.md` |
| 0066 swap-acid | [199](https://github.com/diegoami/neo-angband-tracker/issues/199) `eddbf5b82` | `91f075bdc` | `0d4bd9e26` (PR 281) | #270 | `9eca003a58` | `SWAP-ACID.md` |
| 0067 staircycle | [403](https://github.com/diegoami/neo-angband-tracker/issues/403) `d92e06c3f` | `bd9a36def` | `0d4bd9e26` (PR 281) | #280 | `43dee40ed0` | `STAIR-CYCLE-POISON.md` |
| 0068 text-bufsize (correction to patch 0025) | [287](https://github.com/diegoami/neo-angband-tracker/issues/287) `e7f670fa4` | `60034c84c` | `72dd1c8a3` (PR 284) | #93 | `e41f4c99eb` | `TEXT-BUFSIZE.md` |
| 0069 glow-isglow (correction to 0050) | [213](https://github.com/diegoami/neo-angband-tracker/issues/213) `c316a65b2` | `b5e07827a` | `47be91c95` (PR 285) | #224 | `f2b402c92c` | `GLOW-ISGLOW.md` |
| 0070 iswarded | [198](https://github.com/diegoami/neo-angband-tracker/issues/198) `62242ac26` | `ecfea5064` | `53c17b06c` (PR 291) | #286 | `5454144a62` | `ISWARDED-198.md` |
| 0071 painmsg | [197](https://github.com/diegoami/neo-angband-tracker/issues/197) `8a0c13f71` | `1e0031455` | `6002bfa4b` (PR 292) | #287 | `ffb74e54ad` | `PAINMSG-197.md` |
| 0073 trollswap | [201](https://github.com/diegoami/neo-angband-tracker/issues/201) `4fe83bc81` | `288556d2b` | `3e50961cd` (PR 293) | #289 | `2f3a73f374` | `TROLLSWAP-201.md` |
| 0074 drophole | [136](https://github.com/diegoami/neo-angband-tracker/issues/136) `ca9047877` | `6e1858a71` | `3e50961cd` (PR 293) | #290 | `ac57d5169f` | `DROPHOLE-136.md` |
| 0075 wearskip | [85](https://github.com/diegoami/neo-angband-tracker/issues/85) `5d73e0c21` | `7b97f0f88` | `f7e489157` (PR 295) | #294 | `3735a7e728` | `WEARSKIP-85.md` |
| 0076 lantern | [90](https://github.com/diegoami/neo-angband-tracker/issues/90) `bd4698a85` | `69a62b869` | `63979b54b` (PR 297) | #296 | `8ba3564002` | `LANTERN-90.md` |
| 0077 homebanish | [54](https://github.com/diegoami/neo-angband-tracker/issues/54) `e49ab9480` | `6373f9b8a` | `3573d741e` (PR 299) | #298 | `e24fa8ccab` | `HOMEBANISH-54.md` |

Every carry is a `cherry-pick -x` of one reviewed C-only source commit; the two corrections are
owner-decided amendments to already-carried patches (0025 and 0050), not new sources. The 0064/0065
pair and the 0066/0067 pair each merged from one stacked branch; 0073/0074 likewise (PR 293, two
steps with per-step before-build snapshots and step gates). No owner decision comment was found on
the new source issues, so the carries ran under the standing owner rule (RULES 13: merge verified
PRs after agent evidence review); 0068/0069 corrected owner-flagged rows.

## Verification contract (what is and is not the same as milestone 6)

1. **Unchanged source fixture, fail-before/pass-after on the observable, byte-identical
   controls.** Runtime fixtures are asserted byte-identical to each kit's run pin. **Two
   AddressSanitizer phases ran in this milestone**: for 0068 (text-bufsize — the wide case
   reports the stack-buffer-overflow WRITE the correction bounds, `TEXT-BUFSIZE.md`) and
   for 0070 (iswarded-198): the 0070 defect is a heap-buffer-overflow READ (out-of-level
   `square_istrap`), observable only with assertions off, so both carries' ASan phases use
   NDEBUG like the source kits while the normal phases keep assertions and abort with
   SIGABRT (0070) or report the overflow (0068). The 0070 ASan sites recorded on the
   product (`borg-messages.c:1192`/`:629`) are the source kit's sites (`:1188`/`:625`)
   shifted by the consumer's carried hunks. No other carry in this milestone ran an ASan
   phase; none of the other defects is a memory error.
2. **Rows equal the source's recorded rows value for value** for every carry (including the
   `puff` mode of painmsg-197, which keeps failing in both phases by the kit's own contract, and
   wearskip-85's `nocloak`, which stays 2-action in both phases). No adapted fixture was needed
   anywhere in this milestone.
3. **Provenance byte-wise per module:** before module + source diff == product module, and the
   applied commit is the only `src`/`lib`/build-file difference from its before commit (per step
   for the stacked pairs). 0070/0071/0073/0074/0075/0076/0077 files were otherwise pristine;
   0068/0069 prove the corrected module on top of the patch they amend.
4. **Native gate 965/965 unit + 5/5 frontend** on every fix commit or stacked step tip, read
   numerically; identical hosted numerics on every PR. Tooling tests 129 on every PR
   (milestone 6 ended at 121).
5. **The prior-fixture regression matrix was rerun in full at the v7 tip** — the milestone 6 gap
   is closed. The runner is committed as `tools/regression-matrix.py` (milestone 6 follow-up 2):
   it links every accumulated fixture against the build's core objects, executes each mode, and
   checks the recorded expectation — for the 58 groups with retained local baselines it compares
   the parsed OUTCOME rows value-for-value against the baseline logs and reports byte-identity;
   for the 17 kits whose worktrees are retired (milestone-6 carries and 0064-0067) it fetches the
   fixture at the ledger's evidence pin and drives the kit's own `run-cases.py`. Declared standing
   states are asserted, not weakened: dispel-248 `undead` (losdispel-326 replaced launch values,
   refusal outcome unchanged), dispel-259 `reserve`/`clear` keep failing (the blast scan the
   losdispel correction made reachable moved their absolute values — reproduced exactly, -10627 →
   -10331 and -127 → 169), recallmana-157 `lowmana` stays red with `halfmana` green and
   `restore_mana=0` in both (the 0053 flee interaction, exactly as `verify-recallmana` records
   it). The 0025-era text-multibyte fixture is retired in the matrix: its scenario targets the
   pre-0068 five-argument `borg_what_text` signature and no longer compiles against the tip; the
   text-bufsize kit (0068) covers the same path and is run in its place. The rerun surfaced one
   absolute-count shift, investigated and declared rather than waved through: knockout/paralysis
   records one more pre-kill reaction than its 0049-era baseline (`reactions_before` 0 → 1,
   `reactions_after` 3 → 4) because painmsg-197 (0071) makes more monster-status lines parse fully
   and queue reactions; rerunning the mode against the 0070-after build reproduces the old counts
   (0/3, delta 3) and against the 0071-applied build the new counts (1/4), with the knockout fix's
   own invariant — `delta=3` — preserved everywhere. The matrix asserts the delta and declares the
   shift. **Official run (2026-11-08, tip `3573d741e`, gate 965/965 + 5/5 at the same commit):
   75 groups, 225 cases — 57 local groups run (185 cases; 183 row-compared value-for-value against
   their recorded baselines, of which 181 logs are still byte-identical) + 17 source-tracker kits
   (40 cases, all passing with recallmana's declared standing state reproduced exactly) + 1 retired
   group skipped — zero unexpected mismatches.** The summary is committed as
   `docs/measurement/impact-v7/regression-summary.json`; the runner's recorded rows become the
   baselines for future reruns.

## Frozen v1 cohort: per-patch and cumulative

Per-patch (seeds 11/29/47, cap 20000, wall 120, stall 30, observer builds of before and after):

* **0064/0065 (reverse-flow pair, pair measurement): all three traces diverge** — behavior-active;
  seed 47 is alive in this lineage.
* 0066/0067: byte-identical on all three seeds (all alive; the lineage includes the reverse-flow
  pair, where seed 47 is alive).
* 0068, 0069, 0070: byte-identical on all three seeds (neutral; 0069's torch-lit marking change
  is behavior-local to BORG_LIGHT and the cohort is byte-identical).
* **0071 (painmsg): behavior-active** — seeds 11 and 29 diverge (the four monster status endings
  now queue matched reactions that change later decisions), seed 47 byte-identical; all three
  runs alive and well at the cap. Recorded as observed, attribution clean (the 0070-after cohort
  was byte-identical to its predecessor), not a benefit claim.
* 0073/0074, 0075, 0076, 0077: byte-identical on all three seeds against their actual baselines.

**Cumulative v6 to v7** (`3573d741e6dc3cafa4099fcef8b38fdb32952977` against `b3c825c97c8aef18b252a7fb7d9b7dceef83d586`;
frozen v1 parameters; the v7 rows are the 0077-after cohort run, byte-identical back to the 0071
lineage):

| seed | v6 | v7 | traces |
| --- | --- | --- | --- |
| 11 | alive at cap, level 3 | alive at cap (hp 28, depth 0) | diverge |
| 29 | alive at cap, level 5 | alive at cap (hp 49, depth 2) | diverge |
| 47 | **death by a cave spider, turn 11780** | **alive at cap** (hp 58, depth 1) | diverge |

**Three of three seeds are alive at the v7 tip against two of three at v6.** The repair of seed 47
is located by the per-patch sequence to the 0064/0065 reverse-flow pair, measured as a unit: the
pair's cohort run shows seed 47 alive immediately after it and staying alive through the rest of
the chain; no 0064-only cohort step exists, so no attribution to one patch within the pair is
made (the pair's receipts say the same). 0071 changes seeds 11/29's traces (behavior-active,
alive at the cap). The location is on the v6 trajectory, not an isolation claim for either
patch; three fixed seeds are not statistics, and no survival, progress or population
claim is made in either direction. The cohort lineage restarts at the v7 tip.

## Exclusions and pending items (checked, not counted)

* **spasticscore-147 deferred** (was prepared as 0072; consumer
  [#288](https://github.com/diegoami/better-borg-c/issues/288), source
  [147](https://github.com/diegoami/neo-angband-tracker/issues/147)): the kit's fixture
  choreography was designed for the pristine base where the #146 reverse flow is broken; this
  product carries 0064 in the same function, so the unchanged fixture records zero searches with
  before logs byte-identical to after and demonstrates nothing. The cherry-pick was reverted
  before any PR; no patch entry; deferral receipts on both issues. Carrying it needs a
  0064-compatible product-side fixture (source-lane fixture question).
* **prepreason-234** is not carried: the source states its native fixture is still pending.
* **tapunlife-249** stays masked by patch 0022 (diagnosis in `TAP-UNLIFE.md`, consumer 140);
  **dontreact-14** is unchanged from the milestone 5/6 notes.

## Full-disclosure index: retained errors and corrections

Nothing was weakened for green. Disclosed from this milestone's work: (1) spasticscore-147 was
cherry-picked and its verifier written before the unchanged fixture was found to exercise nothing
on the 0064-carrying product; the deferral (with receipts) happened before any PR and no patch
entry remains; (2) the matrix runner's bring-up needed four recorded corrections before the
official run — `ls-tree` tree-path listing (kit materialization), the kit scenario's working
directory (game data resolves from the binary's directory, so kit binaries are linked into the
build's `game/` like the original observers), companion headers (`t287-variant.h`) now copied
with the fixture source, and the recallmana declaration first written `fail_all` and corrected
to the verifier-recorded state (`lowmana` red, `halfmana` green, `restore_mana=0` in both) after
the first run reproduced the committed standing state; (3) two earlier matrix launches aborted
mid-run before any result was recorded — one at the per-spell baseline log naming
(`both-after-*.log`), one at the retired 0025-era text-multibyte fixture (see contract item 5);
the relaunched run is the official one. Per-carry harness corrections are
disclosed in their own verifier commits and docs (0070: NDEBUG ASan phases, SIGABRT-scoped exit
check, frame-index-free site extraction; 0071: the `puff` keep-failing contract; 0073/0074:
stacked-PR scope check and `floor_items` expectation). Per-carry evidence archives under
`artifacts/` are ignored by Git and kept locally; every consumer issue records pins, commands,
output and limits.

## Review and triage (issue #301)

The milestone 7 review (posted 2026-11-08T19:10:41Z, comment 6725829353) re-executed the gate at
the tip in its own build, reran the **full matrix in its own build (75 groups / 225 cases, 0
failures, 0 differences vs the committed summary including every log SHA256)**, re-ran 9 of 13
carry verifiers to `verified: true` (0068-0077, including 0070's ASan phases), took 0064-0067
through the kit path at their evidence pins with fail-before reproduced from a fresh pre-carry
build (`afdd7921d`), `git apply --check`ed all 13 patch files and reproduced each applied commit's
tree byte-for-byte, made 50 product-vs-kit log comparisons (0 mismatches), and confirmed the
knockout attribution independently. **Verdict: approve, with corrections.**

| ID | Severity | Disposition |
| --- | --- | --- |
| M7-01 | minor-to-major | **Applied**: two ASan phases ran this milestone (0068 and 0070), not one; contract item 1 corrected. |
| M7-02 | minor-to-major | **Applied**: the pair was measured as a unit; the per-patch-attribution wording is replaced by the pair-level measurement statement (headline, cohort section, cumulative paragraph). |
| M7-03 | minor | **Applied**: series.json has **76** entries (13 units this milestone; 0072 never entered); the authoring session's checkpoint receipt on issue 1 said 77 — corrected here and on the processing receipt. |
| M7-04 | minor | **Applied**: `merge_commit` added to the 11 entries 0066-0077 (ledger field was missing; the doc's table was already right). |
| M7-05 | minor | **Applied**: `carried` + `patch-active` labels added to the seven carry consumer issues (#286 #287 #289 #290 #294 #296 #298); `not-carried` added to the deferral #288. |
| M7-06 | minor | **Applied**: the correction target is source #287 / patch **0025** (0118 is an unrelated glyph bug); fixed in the intro, the unit table, the provenance paragraph and both matrix mentions. |
| M7-07 | observation | None needed: deferral technically sound. |
| M7-08 | observation | None needed: scope clean. |
| M7-09 | observation | None needed: matrix arithmetic exact. |
| M7-10 | observation | None needed: knockout attribution confirmed independently. |

**Condition before tagging:** M7-01/02/03/06 corrected in this document and the issue body
(done), M7-04/05 brought up to date (done) — and the annotated tag placed on the runtime
snapshot `3573d741e` (not the docs commit) with a message recording that the full matrix was
rerun. The tag itself was placed under explicit owner authorization (2026-11-08): annotated tag `4.2.6-impact-v7` (object `8425369`) on `3573d741e6dc3cafa4099fcef8b38fdb32952977`, message recording the matrix rerun and the review verdict.

## Tag

The annotated tag `4.2.6-impact-v7` (object `8425369`) was created and pushed on the runtime
snapshot `3573d741e6dc3cafa4099fcef8b38fdb32952977` (runtime commit, not the docs commit),
following v1-v6, after the review's corrections were applied. Its message records that the
full regression matrix **was** rerun at this tip — the first v-tag whose message can say so.
