# Harness

The deterministic headless harness of [borg-rs](https://github.com/diegoami/borg-rs),
copied from borg-rs `e5cb263` (task T1.2, #5) with the same names, arguments
and outputs. borg-rs's `docs/harness.md` (at that pin) is the full manual.

```sh
BORG_IMPL=c    flock /tmp/angband-validation.lock harness/build.sh      # build/c: tree's own C Borg
BORG_IMPL=rust flock /tmp/angband-validation.lock harness/build.sh      # build/rust: this repository's Borg
BORG_IMPL=c    flock /tmp/angband-validation.lock harness/suite.sh smoke harness/out/smoke-c
harness/diff --suite smoke --expect-impl c harness/out/smoke-c harness/reference/smoke-c

# Controls (T1.3): same engine tree and borgh, pinned C Borg overlaid at build time
BORG_IMPL=c-pristine flock /tmp/angband-validation.lock harness/build.sh
BORG_IMPL=c-pristine flock /tmp/angband-validation.lock harness/suite.sh full harness/out/full-c-pristine
harness/diff --suite full --expect-impl c-pristine harness/out/full-c-pristine harness/reference/full-pristine
BORG_IMPL=c-bbc-m7   flock /tmp/angband-validation.lock harness/build.sh
```

## What differs from borg-rs

| File | Change |
| --- | --- |
| `identity.sh` | The Rust build inputs are `angband borg-reborn-rs borg-rs Cargo.toml Cargo.lock harness/measurement`. Here `borg-rs/` is the seam package (see `borg-rs/Cargo.toml`): the imported CMake builds `-p borg-rs` and links `libborg_rs.a`, and the seam re-exports the reborn crate. |
| `run.sh` | Its last step calls `tools/export-death.py`: a natural death in any run, on any arm, writes a basic dossier to `docs/deaths/` (or to `$BORG_DEATH_RECORDS_DIR`), as RULES 12 requires. Repeats are recognised. If the export fails, the run fails. |
| `measurement/rust-overlay.py` | Copies both workspace members (`borg-rs/` seam and `borg-reborn-rs/`) into the build-only overlay the imported CMake builds from when the harness is on. Its instrumentation targets borg-rs's own source files and applies to nothing here. |
| `control-overlay.py` (new) | Writes `build/<impl>.src/`, a copy of `angband/` whose `src/borg/` is the control's pinned C Borg (`controls` in `pins.json`), checked blob for blob. The tree itself is never edited. |
| `build.sh`, `suite.sh`, `identity.sh`, `diff` | Accept the controls `c-pristine` and `c-bbc-m7`. `build.sh` compiles the overlay as CMake `BORG_IMPL=c` into `build/<impl>/`. `suite.sh` maps the executable's and CMake's `c` to the control's name, and the `BUILD_ID` names it. `identity.sh` adds the control's pinned source (commit and tree) to the build-input hash. |
| `suites/full` | Copied from borg-rs: 9 classes × 5 seeds × 100,000 turns. Without the `menu` flag, class indices of 7 and above pick other birth entries (borg-rs `docs/harness.md`), so the `-8-blackguard-` runs play other classes. |
| `run.sh` flag `exact` (T1.4) | Passes borgh's existing `-exact-turns`, so the run stops at engine turn TURNS itself (the per-turn hook in `game-world.c`), not after the first key at or past it. Existing suites keep their semantics; `suites/smoke-exact` and `suites/full-exact` use it. |
| `run.sh` → `tools/depth-stream.py` (T1.4) | After every run, writes `depths.json`: the boundary rule (`exact` or `after-key`), whether the character is alive at the stop, its maximum depth there, and the first engine turn at each depth (the first Borg key after arrival, from `state.log`). An exact survivor that did not stop at the cap fails the run. |
| `check-boundary.py` (T1.4, new) | For a suite run with and without `exact`: keys before the boundary are a prefix of the normal run's keys, survivors stop at the cap (a living character that stopped earlier for any reason but a victory fails), deaths and victories are the same, `depths.json` agrees with `summary.json`, and the suite file matches the manifest's `suite_sha256`. |
| `run.sh -- -load FILE`, `check-extra-load.sh` (M1 re-review R3, S1, S2) | A save given in the extra borgh options is handled like `save=NAME`: copied to `OUT/save/input.save` and passed by absolute path, without `-n`. `config.txt` records its `save_sha256`, plus `keep_load_rng=1` for an extra `-keep-load-rng`. `check-extra-load.sh` (in CI) checks that such a load, given by a relative path, resumes the saved game and plays the same keys as `save=NAME`. |
| `suite.sh` `run_exit` (M1 re-review) | Writes `RUN/run_exit`, run.sh's exit status, for every run. `tools/endpoint.py` scores a run of a `FAILED` suite only if it is 0, and no run of a suite whose binary changed while it ran. |
| `build.sh` `BORG_TEST_FEATURES` (M1 review) | Test-only cargo features for `BORG_IMPL=rust`, built into `build/rust-test/` so `build/rust/` is never a test build; `BUILD_ID` records them. `test-hallucinate` makes perception run while the player hallucinates and asserts on every think that the game's whole RNG is unchanged (CI). |
| `README.md` | This file. |

Compute the primary endpoint from exact-boundary outputs:

```sh
tools/endpoint.py --local c-pristine=harness/out/full-exact-c-pristine \
                  --local c-bbc-m7=harness/out/full-exact-c-bbc-m7 \
                  --depth 5 --turns 100000 --pair c-pristine c-bbc-m7
```

All other files are byte-identical to borg-rs's.

## References

| Directory | What it is | Use |
| --- | --- | --- |
| `reference/smoke-c/` | The smoke suite run by borg-rs's own C build at `e5cb263`: impact v6, the same `angband/` tree as ours. Keys, messages and state, produced on 2026-11-09 | Oracle for `BORG_IMPL=c` here. Our build reproduced it byte for byte, which proves the import (T1.2). Before `measurement/rust-overlay.py` was adapted, the build-input hash was also identical (`d6ada527…`). The adaptation changes that hash but not the C binary |
| `reference/smoke-c-bbc-m7/` | Our own `c-bbc-m7` smoke run of 2026-11-09 (T1.8). It is **self-produced**: no outside oracle runs better-borg-c milestone 7 under borgh | Regression reference: CI checks that the control has not changed |
| `reference/smoke-pristine/`, `reference/full-pristine/` | borg-rs's committed smoke and full references, unchanged from `aa4ad6c` (the zoo's `4.2.6-rs-m0-c`, pristine C Borg) to `e5cb263` | Oracle for the `c-pristine` control. **Checked 2026-11-09: smoke 3/3 and full 45/45 byte-identical** (keys, messages, state). They do **not** match `BORG_IMPL=c`, whose C Borg carries the milestone 2–6 fixes |

A reference is never rewritten. A new tree (an engine re-sync, RULES 22)
gets a new reference directory.

## Not here yet

- The reborn Borg reports each decision to borgh (`borgh_decision`, cargo
  feature `borgh`, which `build.sh` turns on for `BORG_IMPL=rust` through
  `-DBORG_RS_FEATURES=borgh`). It has no `borg.txt` settings to report.
