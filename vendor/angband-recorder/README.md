# Angband Recorder

A Rust recorder plug-in for Angband, with thin C engine hooks. It records human
or bot input and verifies replay by recomputing the game. Borg logic, parity
harnesses, death workflows, named launchers and consumer engine fixes stay in
their own repositories.

The initial engine adapter targets vanilla **Angband 4.2.6,
`42cd156ac9f95ddf4311e88dd7ad462292d6d1d6`**, on 64-bit Linux. The CMake
option is **off by default**. The crate owns capture, replay, ordered input
streams, time wrapping, file preservation, frame encoding and verification.
C translates Angband structures and provides a forked save callback. See
[the comparison](docs/COMPARISON.md), [format](spec/streams-v1.md) and
[credits](CREDITS.md).

```sh
cargo test --locked
cargo clippy --all-targets --features ffi -- -D warnings
python3 tests/round-trip.py --engine /path/to/angband-git --output /tmp/recorder-test
```

The round trip exports the exact vanilla commit, applies only the hook patch,
builds with the C Borg disabled, creates a Human Warrior through the stock test
frontend, plays eight turns, saves and quits, and verifies streams and final
save bytes. It also rejects a changed RNG observation. Scratch stays outside
Git; heavy work takes `/tmp/angband-validation.lock`.

To build manually, apply `patches/angband-4.2.6-42cd156.patch` to that exact
engine, then configure:

```sh
cmake -S /path/to/engine -B /tmp/recorder-build \
  -DSUPPORT_BORG=OFF -DSUPPORT_BORG_RECORDING=ON \
  -DSUPPORT_TEST_FRONTEND=ON -DANGBAND_RECORDER_DIR="$PWD"
cmake --build /tmp/recorder-build -j4
cargo build --locked
```

From the game's working directory, using a private profile and a recorder-enabled
binary:

```sh
/path/to/angband-recorder record /tmp/game-recording -- ./angband -mx11
/path/to/angband-recorder replay --verify /tmp/game-recording /tmp/game-replay
/path/to/angband-recorder list /tmp/game-recording
/path/to/angband-recorder view /tmp/game-recording 25
```

`record` inherits the terminal. `replay --verify` reruns the exact local
executable, whose hash must match. `verify SOURCE REPLAY` only compares already
produced streams. `view` reconstructs recorded screen deltas at a frame; it is
a text viewer, not the planned in-game menu. Existing consumers retain their
richer viewing/branching adapters during migration.

Set `ANGBAND_RECORD_DIR` to capture directly, or also set
`ANGBAND_REPLAY_DIR` to replay into a separate empty output directory. Existing
`BORG_RECORD_DIR` / `BORG_REPLAY_DIR` aliases remain accepted. Recording begins
at process startup. Switching on in an already running game requires a save
and restart; that menu trigger is future work.

Consumers vendor a tagged release and a manifest of commit/tag/file hashes.
Run `tools/check-pin.py PIN VENDOR` before a build. A Rust consumer enables
`angband-recorder`'s `ffi` feature through its **existing Rust staticlib**;
it must not link an additional recorder staticlib. Standalone vanilla testing
links the recorder's single staticlib instead. The C oracle/parity build keeps
the option off.

The owner handover's next features are a single file with a branch tree,
rolling save points, an in-game menu/scrubbing and restart-based branching,
portability, size reduction and human GUI qualification. These are not
claimed by the initial stream extraction. Curated recordings belong in
`diegoami/borg-cemetery`, with release assets only when needed for size.
