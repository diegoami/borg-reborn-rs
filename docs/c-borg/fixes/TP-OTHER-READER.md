# Source250 reader correction: already in the product through patch 0012

Consumer [200](https://github.com/diegoami/better-borg-c/issues/200),
canonical source [250](https://github.com/diegoami/neo-angband-tracker/issues/250)
(tpreader-250), source commit `e890819a6b5ac6e7f7135dcc2b234fcc520b9752`,
evidence head `9ca89f4461e518f28c85962a51aaa6308e0b0158`.

## Outcome: overlap, no new patch

`borg_danger_one_kill()`'s Teleport Other exclusion loop ran to
`ii <= borg_tp_other_n`, one slot past the saved entries. The source250
correction changes that one token to `<`. The product already has exactly
that change: the source67 carry (consumer
[54](https://github.com/diegoami/better-borg-c/issues/54), patch
`patches/0012-tp-other-skip.patch`, applied as
`706fb49b0731ebaf54408c0699b4cd1da1a15929`, merged `cfaff7be7`) fixed the
same line for the same stale-slot read. A cherry-pick of the source250
commit onto `origin/main` (`68765997f`) is empty; no patch is added to
`patches/series.json` as a new unit. `TP-OTHER-SKIP.md` also records that the
writer side of source250 (unbounded append past 255 entries) is unreachable in
native C, so the reader is the whole product-relevant defect.

This document proves the overlap on source250's own fixture rather than
inferring it from the diff.

## Verification

`tools/verify-tpreader.py`, source evidence fixture used unchanged
(`scenario.c`/`borg.txt` byte-identical between the source run pin
`6f6710ccc` and the evidence head; the verifier asserts that). The
`before` build is `origin/main` `68765997f` with only the loop bound restored
to the pristine `<=` (the verifier diffs the two `borg-danger.c` files and
requires exactly that one line); the `after` build is unmodified `origin/main`.
Both link the fixture against their own core objects; the two builds share the
unit-test helper objects, whose sources do not depend on the changed line.

```sh
git remote add source-tracker https://github.com/diegoami/neo-angband-tracker.git
git fetch source-tracker task/tpreader-250 task/tpreader-250-evidence
# before: sed the bound back to <= in src/borg/borg-danger.c, then
cmake -S . -B build/native -DSUPPORT_BORG=ON -DSUPPORT_TEST_FRONTEND=ON -DSUPPORT_X11_FRONTEND=ON
cmake --build build/native --parallel 10 && cmake --build build/native --target OurUnitTestLib
# snapshot build/native as the before build, restore the line, rebuild, then:
flock /tmp/angband-validation.lock python3 tools/verify-tpreader.py \
  --before-build BEFORE_BUILD --before-tree DIR_WITH_PRISTINE_borg-danger.c \
  --after-build AFTER_BUILD --after-tree AFTER_TREE --output artifacts/tp250/scenarios
```

Actual results (`artifacts/tp250/scenarios/result.json`, `verified: true`,
2026-11-08), seed 1000, adjacent hill giant, production
`borg_danger_one_kill()`:

| mode | before (`<=`) | after (product) |
| --- | --- | --- |
| `stale-hit` (one real entry, giant's index one past the count) | exit 0, **0/1**, `danger=0` | exit 0, **1/1**, `danger=179` |
| `in-list` (giant genuinely listed) | 1/1, `danger=0` | 1/1, `danger=0`; log byte-identical |
| `empty` | 1/1, `danger=179` | 1/1, `danger=179`; log byte-identical |

These values equal the source evidence's recorded pristine/corrected values
(`0` -> `179`, controls unchanged), so the product reproduces the source run
value for value.

## Scope and limits

* One constructed arena and one planted stale slot; it proves the reader no
  longer consults an entry beyond the count, not a gameplay frequency or
  survival claim. No behaviour change in the product: this is a record, not a
  change to any game or Borg source.
* No sanitizer phase. The stale slot is inside the allocated array, so
  AddressSanitizer has nothing to report (source evidence agrees); this is a
  logic defect.
* The full native gate was not rerun: no product source changed. Hosted checks
  on the PR gate the tooling and docs added here.

## Exact next work

Consumer 200 is labelled carried/patch-active by overlap with patch 0012 and
stays open until upstream resolves source250 and the correction is
incorporated, per RULES 5. Next uncarried source fixes (not yet adopted):
tapunlife-249 (masked by patch 0022, see consumer 140), stairrange-245,
clockreset-246, drainedive-235, prepreason-234, swapacid-199, spastic-146,
glyphflow-345 and staircycle-403 (the last awaits the consumer 204 correction
boundary).
