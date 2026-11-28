## 2026-11-28: The full reborn-core suite in its own concurrent CI job (#1400)

The engine job ran the full suite (`harness/suite.sh full` on the test build,
its diff against `harness/reference/full-rust`, and the two checks that read
its output) after everything else: 497 s of 14m14s in run 46094205306. Every
PR that can change play, and every push to `main` and `track-p`, waited for
the sum.

### What was done

`.github/workflows/ci.yml` has a new job, `engine-full`, that runs at the same
time as `engine`:

- `engine-full`: checkout (depth 2), the `scope` step moved here unchanged,
  then, only where `scope` says `full=1`, the cemetery death tools (the
  exporter runs when a run dies), the Rust toolchain, apt (`cmake libx11-dev
  libssl-dev`), the test build (`test-hallucinate,test-perception,test-stair-handoff`),
  `suite.sh full` into `harness/out/full-test`, `harness/diff` against
  `full-rust`, `check-perception.py` and `check-no-time.py` on `full-test`.
  Where the suite is skipped the job still passes, with only the checkout and
  the scope step run, so it can be a required check.
- `engine` keeps everything else. It lost `fetch-depth: 2`, the `scope` step
  and the three `full-test` steps; the test build, stair handoff and
  hallucinate smoke stay, as do the smoke diff and both checks.

### The two checks that read both outputs

`check-perception.py` and `check-no-time.py` take any number of directories
and check each run on its own (no cross-run state, verified in the sources),
so each now runs once per job on the outputs that job has:

| check | `engine` | `engine-full` |
|---|---|---|
| `check-perception.py` | `smoke-hallucinate` | `full-test` |
| `check-no-time.py` | `smoke-rust-1` | `full-test` |

The runs covered are the same as with one call on two directories; the only
change is that a failure shows in the job that holds the failing output.

### What was checked

- The workflow parses; the `scope` shell is byte for byte the one of #1389.
- Locally, the `engine-full` commands in order, with the death tools at
  `death-tools-pin.txt` as the `cemetery-tools` action provides them: the test
  build; `suite.sh full` (45 runs, 4 jobs, 456 s, result OK); `harness/diff`
  against `full-rust` (45/45 identical); `check-perception.py` on `full-test`
  (45 runs, 0 differences); `check-no-time.py` on `full-test` (45 traces, 0
  violations). A first attempt failed only because the sibling
  `borg-cemetery` checkout did not match the pin, so the exporter refused
  every death; this is why the job runs `cemetery-tools`.
- Not checked: the timing of a real CI run (it needs the PR's own run), and
  the skipped path (`full=0`), which only a docs-only PR exercises. The
  `if:` conditions are the ones the old steps used.

### What is left, and owner decisions

- **Branch protection (owner):** `engine-full` is a new job name; make it a
  required check next to `engine` once a run shows it. Until then a failure of
  `engine-full` does not block a merge.
- Cost: a second runner slot per run (slots are shared with borg-zoo) and a
  second checkout, toolchain, apt and test build, about 90 s of repeated
  setup. Wall time of a full run should go from about 14 to about 8-9 minutes;
  the PR's own run gives the first measurement.
- Still not decided (as in #1400): a merge queue on `track-p`, and whether
  the `push` runs on `main` and `track-p` need the full suite when the PR run
  already played the same tree.
