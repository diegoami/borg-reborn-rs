# Harness

The deterministic headless harness of [borg-rs](https://github.com/diegoami/borg-rs),
copied from borg-rs `e5cb263` (task T1.2, #5) with the same names, arguments
and outputs. borg-rs's `docs/harness.md` (at that pin) is the full manual.

```sh
BORG_IMPL=c    flock /tmp/angband-validation.lock harness/build.sh      # build/c: tree's own C Borg
BORG_IMPL=rust flock /tmp/angband-validation.lock harness/build.sh      # build/rust: this repository's Borg
BORG_IMPL=c    flock /tmp/angband-validation.lock harness/suite.sh smoke harness/out/smoke-c
harness/diff --suite smoke --expect-impl c harness/out/smoke-c harness/reference/smoke-c
```

## What differs from borg-rs

| File | Change |
| --- | --- |
| `identity.sh` | The Rust build inputs are `angband borg-reborn-rs borg-rs Cargo.toml Cargo.lock harness/measurement`. Here `borg-rs/` is the seam package (see `borg-rs/Cargo.toml`): the imported CMake builds `-p borg-rs` and links `libborg_rs.a`, and the seam re-exports the reborn crate. |
| `run.sh` | Its last step calls `tools/export-death.py`: a natural death in any run, on any arm, writes a basic dossier to `docs/deaths/` (or to `$BORG_DEATH_RECORDS_DIR`), as RULES 12 requires. Repeats are recognised. If the export fails, the run fails. |
| `measurement/rust-overlay.py` | Copies both workspace members (`borg-rs/` seam and `borg-reborn-rs/`) into the build-only overlay the imported CMake builds from when the harness is on. Its instrumentation targets borg-rs's own source files and applies to nothing here. |
| `README.md` | This file. |

All other files are byte-identical to borg-rs's.

## References

| Directory | What it is | Use |
| --- | --- | --- |
| `reference/smoke-c/` | The smoke suite run by borg-rs's own C build at `e5cb263`: impact v6, the same `angband/` tree as ours. Keys, messages and state, produced on 2026-11-09 | Oracle for `BORG_IMPL=c` here. Our build reproduced it byte for byte, which proves the import (T1.2). Before `measurement/rust-overlay.py` was adapted, the build-input hash was also identical (`d6ada527…`). The adaptation changes that hash but not the C binary |
| `reference/smoke-pristine/` | borg-rs's committed smoke reference, unchanged from `aa4ad6c` (the zoo's `4.2.6-rs-m0-c`, pristine C Borg) to `e5cb263` | Oracle for the `c-pristine` control (T1.3, #6). It does **not** match `BORG_IMPL=c`, whose C Borg carries the milestone 2–6 fixes |

A reference is never rewritten. A new tree (an engine re-sync, RULES 22)
gets a new reference directory.

## Not here yet

- The control arms `c-pristine` and `c-bbc-m7`: T1.3 (#6).
- The exact-boundary stop and the depth stream: T1.4 (#7). borg-rs's `borgh`
  at this pin already has an exact-turns option (`opt_exact_turns`); T1.4
  starts from it.
- A playing Rust Borg: T1.5 (#8). Until then `do_cmd_borg` is a no-op, so
  `BORG_IMPL=rust` builds and links, but no Borg starts on that arm.
- The reborn Borg does not yet call borgh's measurement hooks
  (`borgh_setting`, `borgh_decision`, `borgh_rng`). borg-rs's overlay patched
  them into its own sources; ours will call them directly (T1.4/T1.5).
