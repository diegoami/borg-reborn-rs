# Watchdog reads split UTF-8 tails

Consumer tooling [issue220](https://github.com/diegoami/better-borg-c/issues/220).
Reported by read-only borg-zoo milestones-wide
[run42425634696](https://github.com/diegoami/borg-zoo/actions/runs/42425634696),
results branch `results/milestones-wide/42425634696-1/cells.jsonl`.
One of2500 cells failed: M2 source2675d198, seed4333, harness8c23ec0,
74.8296s, no final state, UnicodeDecodeError for0xc3 at1090836.
The selected original cell and full cells-file SHA256 are retained alongside
this report. No zoo data/catalogue was changed.

## Cause and minimal fix

The watchdog re-read the growing trace with strict text decoding before
parse_states could discard its unfinished final line. A read ending inside
an accented name raises UnicodeDecodeError (a ValueError); run catches it as
harness_failure and its finally block reaps the still-live child.

Fix commit **56ccf741c5663a64da61a94d625003c2e52cd4a6** changes only the live
monitor read to `read_text(encoding='utf-8', errors='replace')`. Existing
partial-line rejection then ignores that tail; the next read decodes the
completed character normally. It does not write the trace, change counters,
change watchdog thresholds or modify native/game/observer code. The completed
trace read after process exit remains strict.

The change does not make permanent final corruption acceptable: a run-level
control writes invalid final UTF-8, exits0, and still records harness_failure.
This is a runner correction, not a C Borg defect or a new gameplay carry.

## Deterministic regression and exact native prefix

`python3 -m unittest discover -s tools -p 'test_measure_native.py'`

The new monitor test controls the writer and clock, cuts2/3/4-byte characters
at every interior boundary (six cases), and then completes the character plus
the next STATE record while leaving an ASCII partial record at the new tail.
It asserts the complete records, continued monitoring, no child termination,
and unchanged file bytes. Before the fix:11 tests, six UnicodeDecodeError
subtest errors. After adding the strict-final corruption control:12 tests OK.
The full tooling suite passes97 tests.

The successful native rerun contains Sméagol at the reported location.
Cutting its trace after1090837 bytes reproduces the **exact original error**:
`'utf-8' codec can't decode byte 0xc3 in position 1090836: unexpected end of data`.
The old monitor raises; the fixed monitor accepts the74 complete STATE records
and ignores the unfinished note. This is a controlled prefix read, not another
native life. `watchdog-utf8/native-prefix-proof.json` records it.

## Reader audit

- Ordinary and recorded measurement runs use the same measure-native monitor.
  run-borg-headless.py routes both profiles through run().
- replay-borg.py and tools/borg-takeover.py's live prefix both call that same
  monitor, so they inherit the correction without another decoding policy.
- Their final trace reads, start-recorded-borg.py's final read, and
  borg-recordings.finalize() occur after process exit/reaping. Strict decoding
  remains appropriate. The visible launcher's live autosave copy is binary.
- borg-replay-viewer.py inspects original/offline recordings; it does not
  monitor a live game. Configuration/manifests and postmortem/export readers
  are not the concurrent trace-read path.

## Native verification

M2 source **2675d198e4c3fcb1c69ee63ef25a2b6ac27ab96f**, fixed Human Warrior/default
point buy, defaults, Debug observer, no checkpoint saves, cap100000/wall900s/
stall90s. Local compiler **cc (Ubuntu 15.2.0-16ubuntu1) 15.2.0**, different from
the zoo's Ubuntu24.04/GCC13.3. Binary SHA256:
`092a8715d4d691daf36b95a1c4d4a29d248e988ba9360e3d1dce62af152ad12f`.
Result: **turn_cap100000 alive,43.669s**, depth2/max5, CL10, HP129,
9752 decisions, no error. It started alongside three v1 controls (peak four
concurrent games under one shared validation lock); they finished earlier,
so this does not recreate sustained four-worker load or the exact zoo host.
One successful rerun cannot prove absence of the timing race; the regression
and matching prefix error establish the mechanism and correction.

The native runs loaded the tested working-tree runner before the fix commit;
its SHA256 `e9cd4aa132f234d1f13d3e644ee928aedb022eb1b69418c50b4fca7637a91499`
is the exact measure-native.py bytes in56ccf741c. Recorded harness metadata
therefore names pre-fix parent14eac3b95 and dirty files, rather than relabeling
the runs with a later commit. Full identities/results are in the adjacent JSON.

Frozen v1: pristine sourcee473720ed88a0f0c7ca66815cef3000eb4a38bc3, seeds11/29/47,
cap20000/wall120/stall30, fixed birth and normal checkpoint saves. Same immutable
binary before/after; all complete traces and all STATE records byte-identical,
outcomes all turn_cap. Trace SHA256s in seed order:

- 11: `b329a5584f45c6310750ec0390025bded5265e18b3d65eaacfb7a1d70e384d66`
- 29: `215cfe037c03cd75bd0fedf2122621e16bbd298d7dd18650b272c9041bef9b0b`
- 47: `e27b0d45984936c7f9cb5047c0ce66fd1166dd39f9379a1e077e3082ff600201`

Wall time, runner digest and provenance metadata differ as expected; full
result.json files are not claimed byte-identical. This is output equivalence
on the declared cases, not a survival/performance estimate.

Normal local gate: `flock /tmp/angband-validation.lock python3 tools/verify-native.py`
passed **965/965 plus5/5**. Full tooling **97/97**. Exact final-head hosted checks
and the merged harness commit to pin are recorded on issue220/its PR.

## Reproduction and evidence

From an isolated checkout of the fixed harness:

```sh
flock /tmp/angband-validation.lock python3 tools/measure-native.py prepare \
  --ref 2675d198e4c3fcb1c69ee63ef25a2b6ac27ab96f --observe --output /tmp/utf8-m2-build
flock /tmp/angband-validation.lock python3 tools/measure-native.py run \
  --build /tmp/utf8-m2-build --output /tmp/utf8-m2-4333 --seed 4333 \
  --cap 100000 --timeout 900 --stall-timeout 90 --no-saves
```

Raw builds/binaries/runs/saves, test logs, exact-prefix input and before/after
orchestration are retained in persistent ignored artifacts/watchdog-utf8.
The adjacent archive receipt records SHA256, size and full member readback
verification. Raw evidence is local, not claimed uploaded; curated results and
commands are committed. No independent natural deaths occurred in these runs.

The zoo can pin the merged fixed harness in its own one-line catalogue PR and
rerun the affected cell. No zoo edits are part of this change.
