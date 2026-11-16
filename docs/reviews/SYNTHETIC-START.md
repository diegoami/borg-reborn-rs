# Synthetic start: superseded first-slice checkpoint

The owner review exposed incomplete cave-generation verification. The @1
checkpoint and its results are superseded; use the synthetic P1 milestone
packet and contract @2 for accepted generated-cave evidence.

Owner-directed execution prerequisite for borg-reborn-rs #371 and
borg-cemetery #5. Initial task base: `92404c7e72de4c438ecf1028b17d38b57833d5a6`.
Synchronized review base: `b7d45f171ba7e9b001e39afcaabad84eadc433dd`; its
concurrent validation evidence is preserved without changing runtime inputs.
The review PR names the exact implementation head. This is not a request to
accept a P/R milestone or to close the entire epic.

## Scope and decisions

- Generated dungeon start with a normal level-1 birth kit and no input save.
- Native one-way rules: block ascent/recall, allow further descent. Assess
  this explicit interpretation of the issue's inconsistent single-level wording.
- Strict request validation, actual first-key start verification, versioned
  summary/config identity, visible errors and retained failure evidence.
- No automatic export for synthetic runs, plus a pinned independent guard
  in cemetery's direct exporter and natural-death reconciliation.
- No starting-build catalog/application, null-move exposure evaluation,
  suite runner/report/comparison, or population/improvement claims.

## Runnable review

Use the PR's exact head and the exact `death-tools-pin.txt` cemetery commit.
Set `BORG_CEMETERY_DIR` to that checkout. Build with normal project dependencies.

```sh
BORG_IMPL=rust flock /tmp/angband-validation.lock harness/build.sh
flock /tmp/angband-validation.lock python3 -m unittest discover -s engine/tests -p test_synthetic_start.py -v
python3 -m unittest discover -s tools/tests
python3 -m unittest discover -s scripts/tests
python3 -m unittest discover -s "$BORG_CEMETERY_DIR/scripts/native/tests"
RUN_TIMEOUT_SECONDS=30 harness/run.sh /tmp/review-synthetic-d20 101 0 0 5000 menu,start-depth=20
```

Use fresh output directories. The synthetic test contains six bounded real
attempts (depths 5/10/20 twice, seed 101, Human Warrior, default core/profile),
real death coverage, all-three-log determinism, and negative CLI/receipt checks.
The existing engine CI discovers these tests after building the binary.

Review for town fallback, first-think/first-key generation ordering, native
options and downward transitions, malformed/duplicate/loaded-save requests,
preserved ordinary-run behavior, and synthetic registry pollution through both
launcher and manual exporter paths. Check that timeouts/mismatches are failures
even if a partial summary says alive/dead. Test native-export configuration-only
markers and contradictory markers, not just the new summary field.

Raw logs and toolchain setup are outside Git. Compact evidence and exact runtime
identities are recorded in `SYNTHETIC-START-EVIDENCE.json`. Validation distinguishes
local checks and skips from CI; passing CI is not owner acceptance. Leave both
review branches unmerged until the owner has reviewed the bounded change.
