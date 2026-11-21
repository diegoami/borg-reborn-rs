# Supervised runs and engine checkpoints

Historical oracle note: `endgame` against `harness/reference/endgame` is
expected to report4/5 identical (exit1) with corrected M4 source because
the depth98 messages change under #287. Keys/state match all five.
The corrected checkpoint is `crash-fixes` with its matching reference;
original endgame references and hashed recipes are preserved. See
[saved-character replay](harness.md#saved-character-replay-m4-d).

Every `harness/run.sh` invocation now runs its engine once through
`harness/supervise.py`. A failure is recorded, not retried. `process.json`
keeps the actual child return code, terminating signal when applicable,
timeout/interrupt status, binary hashes before and after, and a stderr
tail. Rust backtraces default to enabled unless the caller already set
`RUST_BACKTRACE`. The engine's `summary.json` is never fabricated.

`RUN_TIMEOUT_SECONDS` optionally limits each engine process's wall time;
0 (the default) disables it. On timeout or forwarded interruption the
supervisor signals only its own exact child PID, then kills that same
child after a two-second grace period if needed. Timed-out and crashed
runs still fail the suite. Other scenarios continue under `suite.sh`'s
existing policy. Only an entirely successful suite receives `DONE`.

## Capture at a fixed cadence

For new runs, for example:

```sh
CHECKPOINT_EVERY=100000 BORG_IMPL=c JOBS=2 \
    harness/suite.sh extended harness/out/c/extended-capture
```

`CHECKPOINT_EVERY=0` is the default and disables capture. A positive value
is an absolute game-turn cadence. The frontend saves at the first
top-level game-command boundary at or after each threshold, outside a
Borg callback or input prompt. A long rest can cross several thresholds;
there is one save at the next safe boundary, not a burst of delayed saves.
Actual and scheduled turns are recorded separately. No keys are injected,
RNG reseeded, or Borg options changed to obtain a capture.

**Saving happens in a forked private engine copy.** The ordinary Angband
saver compacts monster indices and changes save flags. Those mutations
must not happen in the live parent. The single-threaded headless frontend
forks at the safe boundary, closes the child's inherited oracle streams
without flushing them, saves in the child, then waits for that exact PID.
The parent simulation continues from its unchanged memory. On Linux the
saving child also gets a parent-death signal so a killed engine does not
leave its snapshot child behind. This capture implementation requires a
POSIX host; the rest of the harness already uses Unix build tooling.

Each run keeps:

| file | purpose |
|------|---------|
| `checkpoints/checkpoint-NNNNNN-tTTTTTTTTTT.save` | independently loadable engine state |
| `checkpoints/index.jsonl` | flushed records of successfully completed saves: cadence, actual turn, key count, seed, actual race/class, level/depth |
| `checkpoints.json` | supervisor's completed inventory, with file sizes, SHA-256 hashes, source-process result and any incomplete-index errors |
| `process.json` | supervised engine outcome, including failure |

The inventory is also made after a crash or interrupt. Missing files,
truncated index lines and duplicate/invalid records are explicit errors;
orphan or incomplete saves are not automatically accepted. A supervisor
killed outright or lost with the container can leave an unfinished
receipt: its raw files remain available for inspection, but the replay
helper refuses unfinished provenance.

## Replay as a separate scenario

After the source suite has finished (`DONE` **or** `FAILED`), choose a
capture from its completed `checkpoints.json`:

```sh
python3 harness/replay-checkpoint.py SOURCE_RUN CHECKPOINT_NAME NEW_OUT \
    --seed 7300 --turns 200000
```

The arguments in capitals are paths/name chosen from the capture output.
`--turns` is the absolute engine turn limit and must be later than the
captured `game_turn`. Omit `--seed` to use the capture's original seed.

The helper requires a verified C source suite without build overrides,
unchanged binary hashes and a completed capture record whose SHA matches
the save. It copies the selected save into a new fixture directory,
records provenance in `SOURCE.json`, and preserves the source Borg config
flags apart from selecting the saved character. It then runs C twice and
Rust once using their respective verified builds and the same save/seed.
Both strict three-log comparisons must pass to produce `MATCHED`.
Otherwise the matrix gets `FAILED`, with logs and results retained. No
references are installed and main is never moved by this helper.

These are **fresh-Borg replays**, not resumed uninterrupted traces. The
save contains engine state, not the Borg's internal map, queues or other
private memory. Ordinary engine saving may renumber monsters in that
saved copy. The loading frontend then reseeds at the pre-Borg boundary,
as for the existing endgame suite. No traces may be stitched together,
and a matching fresh replay does not turn its failed source into a pass.
Known C faults can receive targeted corrections under the owner
policy in `docs/FORK-CORRECTIONS.md`; replay coverage remains independent.

For manually declared replay suites, `SAVE_ROOT=DIR` selects the directory
used for `save=NAME.save`. The verifier requires an explicit matching
`harness/diff --suite SUITE --save-root DIR ...`; it checks recorded save
hashes against those expected input files. The default is `harness/saves`.

## Verification and boundaries

```sh
python3 harness/regress/supervisor/test_supervisor.py
harness/regress/checkpoint/run.sh
harness/check-no-c-borg.sh
```

Verified in this session:

* Seven supervisor tests: nonzero exits/no retries, next independent run,
  signals, missing summaries, timeout isolation, forwarded interrupts,
  retained captures after failure, truncated-index rejection and immutable
  previous receipts.
* C and Rust smoke runs with capture disabled and enabled: all three logs
  and summaries identical. Two captured C passes and the captured Rust
  pass produce identical saved bytes and capture metadata. Both plain
  implementations also match the existing smoke references (3/3 each).
* C at the depth50 fixture, seed6000, 20k-turn observer control: capture-on
  and capture-off are identical in all three logs.
* Fresh replay of a captured C character: C/C and C/Rust strict matches.
* All three uniformly timed-out smoke scenarios are attempted and the
  suite remains `FAILED`; strict diff refuses it.
* A real C run interrupted after its first completed capture remains
  `FAILED`. The retained capture produces matching C/C/Rust fresh replays,
  and that independent coverage is accepted despite the interrupted source.

These controls establish observation equivalence for the tested states,
not a guarantee about arbitrary undefined C memory. The detection-array
crash and unterminated multibyte text are known C exceptions (tracker #290
and #287). Eight natural Rogue captures now have matching C/C/Rust
references in `harness/reference/captured-rogue`. The 105-case baseline
recheck is in progress. Main and tags stay at their prior positions.

## Corrected extended capture checkpoint (2026-11-06)

At28b8170, corrected extended C/C and Rust/C pass12/12 with all original
limits reached. Thirty fixed1M-cadence captures have identical metadata,
sizes, SHA-256 and direct bytes across both C passes and Rust. C1 copies
and full source provenance are committed in `harness/saves/captured-extended`.
The deepest saved current depth is78; source maxdepth99 does not imply
that every capture is at99. The named natural suite and separate helper
matrices verify fresh-Borg C/C/Rust replays of Warrior78 and Blackguard40,
reaching84/41 over100k additional turns. They are independent engine-state
replays, not restored private Borg state or stitched source traces.
See `validation/2026-11-06-melee/README.md` for exact receipts.
