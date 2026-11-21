# Opt-in recorder (#210, v0.2.0 adoption #351, v0.3.0 adoption #356)

The optional recorder is vendored verbatim from tag `v0.3.0`, commit
`5146ba6ff5187ec2320b9fd310dea466820db160`, with all 55 files sealed in
recorder-pin.json. Cargo and recorder-enabled CMake run the external pin
checker. The crate links through the existing libborg_reborn_rs.a; a second
Rust staticlib is not linked. Recording stays off in the measured harness.

Configure with `-DSUPPORT_BORG_RECORDING=ON`. The consumer adapter in
engine/recorder-hooks.patch applies to the generated tree after the existing
patch series. Pristine engine/angband remains at upstream `574b618c`.
The adapter is regenerated against that actual receiving tree; it preserves
our harness hooks rather than applying the donor's whole vanilla patch.

V2 starts capture/replay at the very top of main, before frontend/config
reads; record_init then attaches real terminal geometry. File-newer queries
and save-menu directory open/read results use the v2 ABI. Saving this
session's current save uses live stat state, because replay's private save
path can differ from the captured input. Forked checkpoints still enter
snapshot mode before saving and never consume or append parent streams.

Capture with ANGBAND_RECORD_DIR (BORG_RECORD_DIR alias accepted). Replay to
a separate output with ANGBAND_REPLAY_DIR and BORG_TOOLKIT_HEADLESS=1 for
borgh. The recorder supplies external inputs and checks recomputed bot keys;
it does not supply RNG state or bot decisions. V2 seals the supported startup
environment/timezone before engine code runs and verifies executable identity.
The native frontend finalizes before _exit; normal quit has a finalizer too.

**Version boundary:** schema `angband-recorder-streams@2` is incompatible
with v0.1.0/schema v1 recordings in both directions. This adoption does not
convert, overwrite or revalidate old sessions. Keep an old recording's exact
old executable, pin and files to replay it with its matching implementation.
The preceding consumer pin was v0.1.0 at
`c4986b26b9fd1c14852df4f1b7393d9030cf812a`.

`harness/check-recorder.py` exercises fresh-game capture/replay, identical
native final saves, forked checkpoints and captured bot actions. It also
loads a captured starting save, preserves the starting/final bytes, removes
only the working path by moving the recorded final save, and replays from
sealed input bytes before saving to live private-session state. Its v2 CLI
verifies both pairs. This bounded headless coverage does not claim GUI,
in-game branching, hot activation or v1 migration support.

The adoption changes source/build identity. Recorder-off play comparisons,
actual new-build death retention and required CI are linked from
[P351-VALIDATION.md](validation/P351-VALIDATION.md. Canonical reports and packets live
in borg-cemetery; raw recording/build proof stays outside Git. No population
experiment is part of this integration.

## v0.3.0 adoption (#356)

A pin update only: the 55 release files replace the 41 of v0.2.0, and the
build label is `borg-reborn-recorder-v0.3.0`. engine/recorder-hooks.patch is
unchanged (the release's vanilla patch did not change), and no strategy,
default frontend flag or recorder-off build changes. Recorder-off builds
never compile the vendor directory. The schema is still
`angband-recorder-streams@2`; v0.2.0 recordings stay readable, but replaying
one needs its own executable (the recorder ties a recording to the hash of
the executable that made it), so keep old builds.

What the release adds for this repository:
- **A closing frame.** Frames are written before each command, so the turns
  after the last one, including a death, were missing from `frames.jsonl`
  when the game left through the frontend's `ar_finish` and `_exit`. `ar_finish`
  now writes one `final` frame of the end state first (`ar_host_final_frame`
  in the vendored host adapter). main-borgh.c already calls `ar_finish`, so
  no consumer code changes. `harness/check-recorder.py` now asserts it in the
  record and replay runs.
- **Rolling native save points, opt-in** (`ANGBAND_SAVE_WINDOW`,
  `ANGBAND_SAVE_EVERY`; vendor `docs/ROLLING-SAVES.md`). They are off unless
  set. Nothing here sets them: making guided player-cemetery sessions select
  128/1000, and the real recorded GUI/core session #356 asks for, are still
  open in #356.
- **Stricter validation:** a recording directory may hold only documented
  entries (vendor `spec/streams-v2.md`).
- `view` reads one screen by frame or turn.

Not checked here: GUI sessions, and recordings made by older builds replayed
with this one (they need their own executable).
