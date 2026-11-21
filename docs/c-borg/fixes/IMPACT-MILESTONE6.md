# Impact milestone 6: 13 carries and one overlap since impact v5 (patches 0051-0063)

Review package for [milestone 6](https://github.com/diegoami/better-borg-c/milestone/6): the 13
verified C bug carries after the milestone5 runtime snapshot `5384f12ed` (50 carries), patches
0051-0063 (consumer issues 242, 245, 247, 250, 252, 254, 257, 258, 259, 261, 263, 266, 267), plus
the source250 adoption that needed no new patch (consumer 200). Runtime snapshot after the last
carry's merge: `b3c825c97c8aef18b252a7fb7d9b7dceef83d586` (PR 271); measured tip `b3c825c97c8aef18b252a7fb7d9b7dceef83d586`
(docs-only commits after the snapshot). Every carry is merged, labelled carried/open+patch-active,
and receipted on its source issue.

## The 13 carries and the overlap

| Patch | Source (issue, commit @ pristine base `e473720ed`) | Applied | Merged | Consumer | Evidence pin | Verification |
| --- | --- | --- | --- | --- | --- | --- |
| 0051 stairrange | [245](https://github.com/diegoami/neo-angband-tracker/issues/245) `c16b3e9d1` | `30bc61d37` | `ae64c39cb` | #242 | `3c9a75e20` | `STAIR-RANGE.md` |
| 0052 clockreset | [246](https://github.com/diegoami/neo-angband-tracker/issues/246) `7cc0ebbd2` | `49f9c19df` | `120ca9916` | #245 | `85ff22540` | `CLOCK-RESET.md` |
| 0053 dangerflee | [156](https://github.com/diegoami/neo-angband-tracker/issues/156) `007d45483` | `26811540d` | `9cb711cd2` | #247 | `809e556b3` | `DANGER-FLEE.md` |
| 0054 recallmana | [157](https://github.com/diegoami/neo-angband-tracker/issues/157) `57d0df063` | `8a604133a` | `b2c9db479` | #250 | `b6bd252e1` | `RECALL-MANA.md` |
| 0055 deskstair | [153](https://github.com/diegoami/neo-angband-tracker/issues/153) `627681006` | `4a88e67f9` | `f0e10ee12` | #252 | `366fd1b08` | `DESPERATE-STAIRS.md` |
| 0056 backaway | [155](https://github.com/diegoami/neo-angband-tracker/issues/155) `df5361ad5` | `86d01e947` | `20d7d4e62` | #254 | `fa0293e58` | `BACK-AWAY.md` |
| 0057 cfghang | [141](https://github.com/diegoami/neo-angband-tracker/issues/141) `45dfc9b1f` | `052129ae5` | `d4f9bb5c9` | #257 | `a3164a61b` | `CONFIG-PARSER.md` |
| 0058 cfgminus | [142](https://github.com/diegoami/neo-angband-tracker/issues/142) `6bc43e71a` | `0ceeaadbb` | `d4f9bb5c9` | #258 | `18f79a3cc` | `CONFIG-PARSER.md` |
| 0059 cfgmissing | [144](https://github.com/diegoami/neo-angband-tracker/issues/144) `8cbee6944` | `e8a50ea9f` | `d4f9bb5c9` | #259 | `1ff9ca1e5` | `CONFIG-PARSER.md` |
| 0060 countsell | [134](https://github.com/diegoami/neo-angband-tracker/issues/134) `026a42d3f` | `93cbfbda0` | `828d5eec2` | #261 | `68b23b2a1` | `COUNT-SELL.md` |
| 0061 drainedive | [235](https://github.com/diegoami/neo-angband-tracker/issues/235) `6d909bf52` | `eb89278d7` | `31e71f087` | #263 | `557c85260` | `DRAINED-DIVE.md` |
| 0062 respois | [184](https://github.com/diegoami/neo-angband-tracker/issues/184) `5e8d4c5e6` | `089acfae4` | `b3c825c97` | #266 | `af01d73cc` | `RESIST-POISON.md` |
| 0063 respoiswar | [185](https://github.com/diegoami/neo-angband-tracker/issues/185) `c056daabf` | `b81495995` | `b3c825c97` | #267 | `6f130a4ee` | `RESIST-POISON.md` |
| overlap (no patch) | [250](https://github.com/diegoami/neo-angband-tracker/issues/250) `e890819a6` | already in 0012 | `eea2eee96` | #200 | `6f6710ccc` | `TP-OTHER-READER.md` (owner: fold into #67, see review M6-03) |

Every carry is a `cherry-pick -x` of one reviewed C-only source commit. Three carry an owner decision
recorded on the source issue (245, 246 and 156, each "option 1"); for the others no owner decision comment
was found on the source issue, so they are carried under the standing owner rule (RULES 13: merge verified
PRs after agent evidence review), not on a per-fix approval.

## Verification contract (what is and is not the same as milestone 5)

1. **Unchanged source fixture, normal phases only.** Each carry ran the source evidence fixture
   unchanged (runtime files byte-identical to the source run pin, asserted) on a before and an after
   build of the same tree, fail-before/pass-after on the observable, with the source's byte-identical
   controls. **No AddressSanitizer phase was run in this milestone** (milestone 5 ran four phases);
   none of these defects is a memory error the sanitizer could observe; the hang fixed by 0057 and the
   crash fixed by 0059 show as non-zero exit codes in their fixtures. The source finding #399 overflow was therefore **not observed** in
   this milestone. It nevertheless occurs during `borg_init()` in every one of these fixtures (it fires
   before any scenario code runs; measured by the milestone 6 review on an AddressSanitizer build of the tip,
   which reported no other sanitizer finding in any mode), so a sanitizer phase would have reported only the
   already-known, separately owned #399 overflow and nothing about the 13 defects themselves.
2. **Rows equal the source's recorded rows value for value** for every carry. One exception is
   disclosed below (0054).
3. **Provenance** is proven per module: before module + source diff == product module, or, for patches
   that stack on an earlier patch in the same function (0058, 0059, 0063), the source change merged with
   `git merge-file` onto the before module. Each verifier also asserts that the carried file is the only
   `src`/`lib`/build-file difference from its before commit.
4. **Native gate 965/965 unit + 5/5 frontend** on every fix commit or stacked tip, read numerically;
   identical hosted numerics on every PR. Tooling tests 121 on every PR (milestone 5 ended at 97).
5. **Not done:** the accumulated 154-case, 48-group prior-fixture regression matrix was **not rerun** at the
   milestone 6 tip, and its runner is not committed (it lived in a retired flowcache worktree), so it
   cannot be rerun from this repository. The matrix result on file
   (`docs/measurement/flowcache-404/regression-summary.json`) is the 0049-era run, i.e. *before* the
   milestone 6 carries; it was not rerun per carry across milestone 6, and this document previously
   described milestone 5 inaccurately on that point. The stacked PRs (cfg trio, respois pair) ran the
   native gate once on the tip, not per patch.
   **Spot-checks performed by the milestone 6 review (not by the author of this package), run against
   `b3c825c97`:** mapwall-212, detectcarry-219, calllight-231 and munchkin-244 (both modes; 0051 touches the
   same munchkin function) pass, and the dispel family reproduces the documented standing states exactly
   (dispel-248 `main` 0/1 as its asserted change, `undead` 1/1; dispel-259 `reserve` and `clear` 0/1 as the two
   preserved expected failures; losdispel-326 passes). These cover `borg-think-dungeon.c`, `borg-caution.c`
   and the map/detection/light paths only; the remaining groups of the 48 were not rerun, no regression was
   found in those that were, and that the rest is unaffected is a judgement, not a measurement.

## Layered interaction disclosed (0054 vs 0053)

The unchanged source fixture for 0054 (recall-wait mana) stays red on the product, before and after, with
identical logs: since 0053 an ordinary Borg in excessive danger flees via desperate stairs before it reaches
the recall-wait line. This is asserted in the verifier. Fail-before/pass-after is shown on an adapted
fixture that changes exactly one source line (`ready_morgoth` 0 to 1); its rows equal the source's pristine
rows. The corrected line is reachable only when the flee branch is skipped. The source lane was told.

## Frozen v1 cohort: per-patch and cumulative

Per-patch (seeds 11/29/47, cap 20000, wall 120, stall 30, observer builds of before and after):

* identical traces on all three seeds: 0051, 0060, 0061; **not run on the cohort:** 0052 (the threshold is
  out of reach of a 20000-turn cap), 0054 (the cohort Warriors have no mana), 0057-0059 (valid default
  configuration) and 0062/0063 (not measured; a shopping divergence is possible and not excluded).
* **0053 (excessive-danger flee) changed an outcome:** seed 29 diverges and later dies to a cutpurse at
  game turn 15149 (was alive at the cap). Divergence starts at the corrected branch fleeing a floating eye.
  Investigation: [#249](https://github.com/diegoami/better-borg-c/issues/249).
* 0055 (nearest desperate staircase): seeds 29 and 47 traces diverge, outcomes unchanged.
* **0056 (back-away per grid) changed two outcomes:** seed 29 is saved (alive at the cap), seed 47 is lost
  (dies to a cave spider at turn 11780). Investigation: [#255](https://github.com/diegoami/better-borg-c/issues/255).

**Cumulative v5 to tip** (`b3c825c97c8aef18b252a7fb7d9b7dceef83d586` against `5384f12ed`, observer builds, same parameters; archive
`artifacts/m6-measurement.tar.gz`, sha256 `f5bd1833c7371b06ca147d0fb778412918557dc3ea7dfd88848ab7e8d6e66374`, 55151447 bytes, 95 members, build/source trees and saves
excluded; ignored by Git, kept locally):

| seed | v5 | tip | traces |
| --- | --- | --- | --- |
| 11 | alive at cap, level 3 | alive at cap, level 3 | diverge |
| 29 | alive at cap, level 4 | alive at cap, level 5 | diverge |
| 47 | alive at cap, level 3 | **death by a cave spider, turn 11780** | diverge |

All three traces diverge from the v5 lineage. **Two of three seeds are alive at the tip against three of
three at v5.** The loss of seed 47 is attributed by the per-patch runs to 0056 (seed 47 was alive immediately
before it and dies immediately after); seed 29's intermediate death under 0053 is repaired by 0056. Three
fixed seeds are not statistics: no survival, progress or population claim is made in either direction, and
the cohort lineage restarts at this tip. The attribution is to 0056's contribution on the 0053-0055
trajectory, not to 0056 in isolation: 0055 had already diverged seed 47's trace (outcome unchanged), so an
interaction between 0055 and 0056 is not excluded by a single frozen seed (milestone 6 review, M6-05).

## Exclusions and pending items (checked, not counted)

* **Ready, not yet carried:** spastic-146, glyphflow-345 and swapacid-199 (verified fixtures; next milestone).
* **prepreason-234** is not carried: the source states its native fixture is still pending.
* **staircycle-403** awaits the consumer 204 correction boundary; **tapunlife-249** stays masked by patch 0022
  (diagnosis in `TAP-UNLIFE.md`, consumer 140); **dontreact-14** is unchanged from the milestone 5 note.
* Related but separate: issues [249](https://github.com/diegoami/better-borg-c/issues/249) and
  [255](https://github.com/diegoami/better-borg-c/issues/255) (cohort deaths, diagnosis only);
  [235](https://github.com/diegoami/better-borg-c/issues/235) (D0009 starvation) is unrelated to these carries.

## Full-disclosure index: retained errors and corrections

Nothing was weakened for green. Disclosed mistakes: (1) the first source250 attempt was an empty
cherry-pick, found and proven as an overlap rather than force-applied; (2) a verifier reused a build left at
the fixed state as its "before" tree (0062), the assertion rejected it and the tree was rebuilt (recorded in
`RESIST-POISON.md`); (3) a generator wrote `MODES = ('main')` (a string, not a tuple) and the verifier
failed on it before any claim was made; (4) the stepwise cfg run failed its provenance step for 0058 because
the plain source diff does not apply on top of 0057, and the three-way-merge proof replaced it; (5) a stray
test call created a malformed duplicate issue ([#265](https://github.com/diegoami/better-borg-c/issues/265),
closed as not planned) and the body of [#269](https://github.com/diegoami/better-borg-c/issues/269) first
quoted a wrong source hash, corrected with a comment; (6) one PR command was rejected by the shell for a
heredoc and rerun from files, with nothing partially applied. Two clarifications from the review (M6-04):
the applied diff of 0058 differs from its source diff by the 0057 hunk it stacks onto, and the merge-file proof
is the authority for it, so a reviewer diffing 0058 against source 142 should expect that difference; and the
body of #269 names "patch 0065", a placeholder ordinal for a carry not in this milestone. Per-carry evidence archives under `artifacts/`
are ignored by Git and kept locally; every consumer issue records pins, commands, output and limits.

## Review and triage (issue #273)

The milestone 6 review (posted 2026-11-08T11:24:54Z) re-executed every carry from its pinned before commit
and reports: all fixture rows equal the source kits' rows (0 differences), all nine controls byte-identical
by SHA256, all five merge-file proofs reproduce the product module, all 13 patch files apply to their exact
pre-carry trees, native gate 965/965 + 5/5, tooling 121, the 0054/0053 interaction and the adapted fixture
faithful, the attribution of the cohort change sound, and no carry to be reverted. **Verdict: approve, with two
documentation corrections and one verification condition before the tag.**

| ID | Severity | Disposition |
| --- | --- | --- |
| M6-01 | minor | Applied: contract item 1 reworded (not exercised becomes not observed, with the measured fact). |
| M6-02 | minor-to-major | Applied in option (b) form: contract item 5 corrected and the review's spot-checks recorded with attribution; the full matrix is **not** rerun and its runner is still uncommitted. |
| M6-03 | minor | **Resolved by the owner:** source issue 250 carried an unanswered owner question (posted 2026-11-08T11:04:53Z: keep tpreader-250 as a separate pair, or fold it into #67?). The consumer disposition (overlap, no new patch) was reached before that question was answered; the owner then chose **fold into #67**, relayed on source 250 and #67 on 2026-11-08. Closing source PRs #380/#381 and the writer-side triage are left to the source lane. |
| M6-04 | observation | Applied: two clarifications added to the disclosure index. |
| M6-05 | observation | Applied: the 0055/0056 interaction caveat added to the cumulative-run paragraph. |

No external verdict beyond this review exists. The tag `4.2.6-impact-v6` (annotated) was created and pushed on the runtime snapshot `b3c825c97c8aef18b252a7fb7d9b7dceef83d586` after the owner's answer on source 250, following v1-v5 (runtime commit, not the docs commit). Its message records that the full regression matrix was not rerun.

## Exact next work

1. Done: owner answer on source 250 (fold into #67); tag `4.2.6-impact-v6` pushed on `b3c825c97`.
2. Commit the regression-matrix runner as a tool before milestone 7 so the matrix can be rerun (follow-up).
3. Next carries: spastic-146 and glyphflow-345 (reverse-flow pair), swapacid-199; recheck the source board;
   diagnose 249 and 255 before any strategy change.
