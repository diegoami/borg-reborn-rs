# harness/

*Ours (docs/SPEC.md §4). The scripts started as copies of borg-rs's harness
in M1 and were re-founded on our own build in milestone E (TE.4, TE.5):
no part reads borg-rs. Everything runs the Rust Borg, the only
implementation built here (SPEC §6). Heavy runs take
`flock /tmp/angband-validation.lock` (RULES 17).*

```sh
BORG_IMPL=rust flock /tmp/angband-validation.lock harness/build.sh
flock /tmp/angband-validation.lock harness/suite.sh smoke harness/out/smoke
harness/diff --suite smoke --expect-impl rust harness/out/smoke harness/reference/smoke-rust
```

| File | What |
| --- | --- |
| `build.sh` | Builds `engine/` (docs/ENGINE.md) into `build/rust/game/angband` and writes `build/rust/BUILD_ID` (source and binary hashes, `engine_upstream`, compilers). `BORG_TEST_FEATURES=F` builds a test variant into `build/rust-test/`. The zoo calls it (docs/ZOO-INTERFACE.md) |
| `identity.sh` | Build identity: the inputs (`engine/` without its tests, `borg-reborn-rs/`, `Cargo.toml`, `Cargo.lock`), their source hash, and the implementation a binary carries |
| `run.sh` | One deterministic run: `run.sh OUTDIR SEED RACE CLASS TURNS [FLAGS] [-- borgh options]`, with the flags `menu`, `exact`, `random-birth`, `finalsave`, `living-save`, `terminal-save`, `keep-load-rng`, `key-limit=N`, `save=NAME`. It fixes the environment (`TZ=UTC`, `C.UTF-8`), gives the Borg its profile and run directory, and writes the run's `config.txt`. Then `tools/depth-stream.py` (`depths.json`) and `tools/export-death.py` (a death's dossier) |
| `suite.sh` | Runs a suite file of `suites/` in parallel into `OUTDIR` with a manifest (suite and binary hashes at start and end), `RUN/run_exit` and `DONE` or `FAILED` |
| `diff` | Compares two suite outputs, or an output and a reference: keys, messages and state logs byte for byte, and `summary.json` (fields added later, such as `game_options`, only when both sides have them) |
| `check-boundary.py` | The exact-boundary contract, a suite run with and without `exact` (below) |
| `check-extra-load.sh` | A save loaded through `-- -load` resumes like `save=NAME` |
| `make-reference.sh` | Makes a reference directory from a suite output. Its `REFERENCE` file takes the build identity and each run's configuration from the suite's manifest; it refuses a suite run with overrides or built from uncommitted inputs (E review, finding 8) |
| `zoo-check.py` | Runs borg-zoo's own adapter (at our pin) against this checkout (docs/ZOO-INTERFACE.md) |
| `supervise.py` | Runs the game as a supervised child (wall and stall timeouts); `process.json` records its outcome |
| `borg.txt` | The C Borg's configuration file. The Rust Borg does not read it; it stays because the zoo copies it |
| `profiles/` | The Rust Borg's profiles (`reborn-default.txt`), docs/ARCHITECTURE.md *Parameters* |
| `suites/` | `smoke`, `full` (9 classes × 5 seeds × 100,000 turns, race = (class + k) % 11), and their `-exact` variants |
| `reference/` | `smoke-rust`, `full-rust`, `smoke-exact-rust`: the Rust Borg's outputs at the commit their `REFERENCE` names; regression references, not oracles. CI compares smoke and smoke-exact |

## The run contract and the logs

| Part | What |
| `run.sh` flag `exact` (T1.4) | Passes borgh's existing `-exact-turns`, so the run stops at engine turn TURNS itself (the per-turn hook in `game-world.c`), not after the first key at or past it. Existing suites keep their semantics; `suites/smoke-exact` and `suites/full-exact` use it. |
| `run.sh` → `tools/depth-stream.py` (T1.4) | After every run, writes `depths.json`: the boundary rule (`exact` or `after-key`), whether the character is alive at the stop, its maximum depth there, and the first engine turn at each depth (the first Borg key after arrival, from `state.log`). An exact survivor that did not stop at the cap fails the run. |
| `check-boundary.py` (T1.4, new) | For a suite run with and without `exact`: keys before the boundary are a prefix of the normal run's keys, survivors stop at the cap (a living character that stopped earlier for any reason but a victory fails), deaths and victories are the same, `depths.json` agrees with `summary.json`, and the suite file matches the manifest's `suite_sha256`. |
| `run.sh -- -load FILE`, `check-extra-load.sh` (M1 re-review R3, S1, S2) | A save given in the extra borgh options is handled like `save=NAME`: copied to `OUT/save/input.save` and passed by absolute path, without `-n`. `config.txt` records its `save_sha256`, plus `keep_load_rng=1` for an extra `-keep-load-rng`. `check-extra-load.sh` (in CI) checks that such a load, given by a relative path, resumes the saved game and plays the same keys as `save=NAME`. |
| `BORG_PROFILE=FILE`, `params.json`, `trace.jsonl` (T2.4, Rust arm) | run.sh gives the reborn Borg its profile (default `harness/profiles/reborn-default.txt`: `name = value` lines in borg.txt's syntax) and its run directory. `config.txt` records the profile and its SHA-256. The Borg writes `params.json` (the parameter table: name, unit, range, default, value, evidence) and `trace.jsonl` (one line per think: turn, mode, goal and budget, candidates with scores, choice, invariant violations, profile hash). The trace is written after each decision and never changes play. `tools/export-death.py` keeps `params.json` and the trace's last 200 lines (`trace-tail.jsonl`) in the dossier, with `profile_sha256` and `decision_trace` in `record.json`. CI checks both, in the run outputs and in the dossiers |
| `BORG_OBSERVATION_LOG=FILE`, `BORG_WORLD_LOG=FILE`, `BORG_KNOWLEDGE_LOG=FILE` (T2.1–T2.3, Rust arm) | The reborn Borg appends its `Observation` of every think to the first file, one JSON line each (a few hundred KB per think, so for short runs), and its world model's digest after that think to the second. It writes the game's static data (`Knowledge`, one JSON line) to the third at its first think. All are off by default, and recording does not change play. CI records a 3,000-turn run and checks: every observation round-trips; replaying the observations without the game rebuilds the live world after every think; the knowledge matches the game data; and the blows computed from it equal the engine's on every think (`BORG_OBSERVATION_LOG_CHECK=… BORG_WORLD_LOG_CHECK=… BORG_KNOWLEDGE_LOG_CHECK=… cargo test recorded`) |
| `suite.sh` `run_exit` (M1 re-review) | Writes `RUN/run_exit`, run.sh's exit status, for every run. |
| `check-characters.py` (E review, finding 1) | Run by `suite.sh`: each run played the race and class its suite line asks for (the names at those indices in `p_race.txt` and `class.txt`). Every suite line has the `menu` flag; without it, the birth screen could leave another character. A mismatch fails the suite. |
| `build.sh` `BORG_TEST_FEATURES` (M1 review) | Test-only cargo features for `BORG_IMPL=rust`, built into `build/rust-test/` so `build/rust/` is never a test build; `BUILD_ID` records them. `test-hallucinate` makes perception run while the player hallucinates and asserts on every think that the game's whole RNG is unchanged and that hallucination changes nothing observed. `test-perception` (T2.5) checks each think's observation and world against the engine through other routes and writes `RUN/truth.jsonl`; `tools/check-perception.py` reads it. CI builds both together and runs smoke and full with them. |

## References are regenerated only on purpose

A change that alters the Borg's keys makes the references differ. A PR that
means to change play says so and regenerates them with `make-reference.sh`. A
PR that does not must reproduce them.
