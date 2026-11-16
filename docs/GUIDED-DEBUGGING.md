# Guided Angband debugging

Owner workflow, 2026-11-15 (RULES 26): the agent launches a recorded game;
the owner plays or gives errands, reports a problem and saves if possible;
the agent collects the evidence and tries to reproduce the problem itself.
The owner should not have to locate files, reconstruct command lines or
diagnose the Borg.

## Evidence ownership

Canonical incident/death reports, reusable scenarios and behavioral trial
comparisons are authored and published in borg-cemetery. This repository holds
implementation, tests and fixture/case pointers, not duplicate gameplay reports.
Original player recordings and working saves may remain in player-cemetery;
case-specific alternative plays and trials use Cemetery's existing tools and
case storage. Preserve original hashes/history when relocating report pages.

## Before play

The agent prepares a unique session directory outside tracked source files.
It records the actual directory and process ID in the conversation and in
the session manifest so another session can find them.

Preserve:

- The exact commit, uncommitted source diff, engine and recorder pins,
  executable hash and relevant build options. Keep a frozen executable
  and its game data available at the path used by the recording, so a
  later rebuild does not replace the executable needed for replay.
- The starting save (if loading), character/save name, effective profile
  and Borg settings, environment overrides and launch arguments. Copy an
  existing character into private session directories; do not experiment
  on the owner's only save. Continuing that copy later must use the same
  session save directory and character name.
- Recorder streams, stdout/stderr, `trace-core.jsonl` and errand outcomes.
  Use the core actually being investigated, normally `BORG_CORE=borg-rs`.
  Record any initial `BORG_ERRAND`, including an explicitly empty value.

The current integration uses the pinned optional recorder described in
[ANGBAND-RECORDER.md](ANGBAND-RECORDER.md). Build a separate X11 executable
with `SUPPORT_X11=ON` and `SUPPORT_BORG_RECORDING=ON`, under the shared
`/tmp/angband-validation.lock`. Set `ANGBAND_RECORD_DIR` to a new recording
directory, `BORG_RUN_DIR` to the session diagnostics directory, and
`BORG_ERRAND_OUT` to the session outcome file. Capture stderr as well.
Pass `-dsave=...` and the other private directories **before** `-uNAME`:
Angband resolves the save filename when it parses `-u`.

The regular watch launcher is not a recording launcher: setting a recorder
environment variable does not enable recording in a binary built without
the feature. `harness/play.sh` is the older player-in-town workflow, not a
ready-made guided recorder for errands over the vendored core. Do not
silently substitute it for this workflow.

Before calling a session replayable, qualify the actual GUI/core route with
a short recording and replay, including the intended fresh birth or loaded
save, an errand, stopping the Borg, and saving/quitting. The pinned v0.1.0
recorder has a verified native headless integration; that alone does not
qualify every human GUI interaction. Capture from process startup; turning
recording on in an already running game is not supported by this pin.
Verify the effective save path before substantial play.

## Recent saves and quick inspection

Owner follow-up after the PlanMage wild-cat death (#350): keep recent native
save points with the recording in player-cemetery. Record whether retention
is actually supported and enabled by the frozen executable. The recorder-owned
implementation is [angband-recorder#7](https://github.com/diegoami/angband-recorder/pull/7);
consuming it requires a tagged release and a dedicated pin update. Do not
silently modify the vendored recorder or assume environment settings enable
hooks missing from an older binary.

For a supporting build, use ANGBAND_SAVE_WINDOW=128 and ANGBAND_SAVE_EVERY=1000
before process startup. Its recording/checkpoints/index.json identifies recent
living saves and periodic older points, with frame/turn/input provenance and
hashes. Keep snapshots, index, plan/configuration and frozen runtime together.
A final/dead save must not replace the last living checkpoint.

Use `angband-recorder view RECORDING FRAME` for read-only inspection; the
new reader also accepts `view RECORDING --turn TURN`, without running the
Borg through the earlier game. Select an exact frame when several command
boundaries share a turn. Copy the nearest earlier native save into a new
private profile and preserve loaded RNG. First check whether the original
build reproduces the reported failure from that point, then compare the fix.
Loading restores the engine but starts a fresh Borg, so this is a saved-position
counterfactual rather than verified replay of the parent's arbitrary suffix.
The PlanMage case establishes both: the full original replay verifies, and
four nearby saved-position original controls die while fixed controls survive.
Replays and continued segments of that life are not new independent samples.

## When something looks wrong

The owner tells the agent what happened and what was expected, for example:

> `/ladder` is in a corner and the turn counter stopped. I expected it to
> keep exploring. I have not closed the window.

If the game responds, stop the Borg with a key and use **Ctrl-S** to save.
Report whether saving succeeded or displayed an error. **Ctrl-X** saves
and quits; it is not necessary to quit just to collect a save. The
recording preserves any input used to stop or save the game.

The agent records an incident marker with the time, errand, observed and
expected behaviour, and the nearest recorder frame/turn when available.
It finds the running session's actual save path and copies a successful
save into the incident directory before further play can overwrite it.
Check modification time, file contents/character description and the
recorded state; a matching name alone does not identify the right save.
Confirm which character, level/depth and point in the session were saved.

If saving fails or input is ignored, the agent preserves the live process
first: screenshot or recorded terminal frame, last flushed inputs and
decisions, stderr, process state and, where available, a debugger stack.
Distinguish a stopped Borg, an unanswered prompt, repeated actions without
world turns, and an actual hang using runtime evidence. Do not insist on
saving, restart repeatedly, or terminate the process before collecting
available evidence. Record incomplete capture honestly if it cannot be
finalized cleanly. The recording and starting save still provide a lead-in
even when the incident itself cannot be saved.

## Agent reproduction and repair

1. Preserve the original session and incident. Replay or load only into a
   separate working directory. Use the original executable and settings
   for the first attempt, before changing the code.
2. Prefer replaying the recorded lead-in. A native save preserves game
   state, but not necessarily the Borg's map, goals, errand queue or other
   process memory. Loading a save and reissuing the errand is a useful
   second experiment, not proof that the original state was reconstructed.
   It can also be necessary to replay from before the unexpected action.
3. Use the recorder CLI's `replay --verify` or stream verification where
   supported. Keep replay outputs separate. Report an exact match,
   divergence (and first mismatch), or inability to reproduce; do not call
   an unverified replay deterministic. See
   [CORE-TRACE.md](CORE-TRACE.md) for the decision trace's fields and limits.
4. Locate the first decision or engine interaction that differs from the
   expected behaviour. Inspect the trace and engine state; instrument or
   attach a debugger to a reproduction when useful. Static code findings
   are hypotheses until the recorded behaviour supports them.
5. Make the smallest supported fix, add a bounded regression check using
   the relevant fixed start or decision fixture, and compare before/after
   behaviour. Follow the normal task/PR, engine-integrity and death rules.

Recorder defects belong in `diegoami/angband-recorder`: reproduce the
problem there and open a PR in that repository (owner, 2026-11-15). Do not
silently patch the vendor copy or add an engine workaround here. Consuming
an updated recorder is a separate pin-update change in this repository.

The agent reports the evidence location, reproduction result, diagnosis,
change, checks and remaining uncertainty. Preserve the original recording
and unique supporting evidence; raw sessions and saves stay outside Git.
Watched sessions are not population measurements or harness runs under
RULES 12. Do not automatically publish or upload them to borg-cemetery.
