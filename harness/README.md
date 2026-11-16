# harness/

*Ours (docs/SPEC.md §4). The scripts started as copies of borg-rs's harness
in M1 and were re-founded on our own build in milestone E (TE.4, TE.5):
no part reads borg-rs. Everything runs the Rust Borg, the only
implementation built here (SPEC §6). Heavy runs take
`flock /tmp/angband-validation.lock` (RULES 17).*

## Synthetic generated-depth start (issue #371, synthetic P1)

```sh
RUN_TIMEOUT_SECONDS=30 harness/run.sh /tmp/synthetic-d10 101 0 0 5000 menu,start-depth=10
```

Contract `borg-reborn-rs/synthetic-start@2` creates a normal level-1 birth
character with its ordinary starting kit, then completes the native dungeon transition before either Borg activates. No input run or save is needed. Race/class and
profile keep their existing meanings; `menu` is recommended for explicit indices.
`-- -start-depth N` is also supported and normalized into recorded flags.

**P1 birth-kit normalization:** the kit is handed out under the ordinary birth
options before the frontend enables one-way options. This is not the engine's
native kit for a character born with no-recall already enabled. In particular,
Warrior `class.txt` conditionally omits its Word of Recall scroll under native
`birth_no_recall`; P1 birth cells retain that ordinary-kit scroll. The reviewed
bank consumes it ineffectively at turn 3 (one refusal per birth cell). This
existing difference is declared for P1 so the independently repeated streams
remain valid; it is not an effective escape. Changing to a native one-way birth
kit requires a versioned contract and a fresh bank/evidence comparison. The
observed-tier projection has its separately declared inventory/knowledge rules.

Depth is a positive decimal integer below the engine's `max_depth` (the shell
also bounds its input to 1..999). Zero, malformed/overflowing values, duplicates,
loaded saves and random birth are refused. Unsupported build/kit options are
refused by the frontend; this slice supports only the normal `birth` kit.

The dungeon is **one-way**, not confined to a single level. Native
`birth_force_descend` blocks upstairs; `birth_no_recall` blocks recall for these
non-winning birth characters. Downstairs and other native downward transitions
remain legal. The requested depth is the starting depth, not a promise about
the final depth. No game rules or Borg decision rules are patched. A Borg may
still attempt an unavailable escape or stop; that is observable play, not a
successful escape or a harness failure to be relabelled as survival.

The engine summary adds `synthetic: true`, `synthetic_contract`, `start_depth`,
`initial_depth` (verified at the first Borg key), `start_build: "birth"`, and
`dungeon_mode: "one_way"`. The first `state.log` row must show `dN/N`.
`config.txt` records the same contract/request with `synthetic=1`; the full
native options remain in `summary.json.game_options`. Ordinary runs retain
their existing summary format. Build/source/binary identity remains in
`build/rust/BUILD_ID`; pin that build, the profile/configuration and seed for
repeatable execution. The horizon is the existing absolute engine-turn cap,
including startup; use `exact` only when that existing stop contract is wanted.

Generation/start mismatch, failure before start verification, unsupported
capability, nonzero child exit and timeout fail visibly. The launcher checks the
engine receipt before postprocessing. Keep `process.json`, stderr and any
partial logs; a failed receipt or partial summary is not a gameplay outcome.
The suite runner must still check launcher/process status and terminal evidence.

Synthetic runs skip automatic death export. The pinned cemetery toolkit also
rejects synthetic markers in summaries, configuration or flags on direct export,
and excludes those deaths from natural reconciliation. They remain experimental
results; no synthetic death is implicitly promoted to a natural dossier.

Bounded checks: `flock /tmp/angband-validation.lock python3 -m unittest discover
-s engine/tests -p test_synthetic_start.py -v`. They run two 5,000-turn attempts
at each of depths 5/10/20 with a 20-second child wall limit, compare all three
logs, exercise a real death, and test invalid requests and rejected receipts.
These tests are included in the existing engine CI job. Raw outputs stay outside
Git. Run-derived character builds, null-move exposure diagnostics, suite
generation/classification and build-impact reports are later slices of
#371 / cemetery #5, not capabilities of this contract.

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
| `make-reference.sh` | Makes a reference directory from a suite output. Its `REFERENCE` file takes the build identity and each run's configuration from the suite's manifest; it refuses a suite run with overrides, built from uncommitted inputs, or whose runs lack one identical profile identity, before writing anything, and builds the reference in a temporary directory (E review and M2 review, finding 8) |
| `zoo-check.py` | Runs borg-zoo's own adapter (at our pin) against this checkout (docs/ZOO-INTERFACE.md) |
| `supervise.py` | Runs the game as a supervised child (wall and stall timeouts); `process.json` records its outcome |
| `play.sh` | A town session (#191): `play.sh OUTDIR [CHARACTERS]`. The X11 build (`watch-borg.sh --build-only`), `BORG_TOWN_PLAYER=1` (the player has the controls in town, the Borg in the dungeon), the state records (`state.jsonl`), the object log (`objects.log`), the trace, then `feedback.sh`. Not a measured run. Needs a display, X11 bitmap fonts (`xfonts-base`) and a UTF-8 locale |
| `feedback.sh` | The end-of-session feedback: `feedback.md` and `corrections.jsonl` (`{"line": N, "text": ...}`, N an object log line). Copied from borg-rs |
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
| `BORG_PROFILE=FILE`, `params.json`, `trace.jsonl` (T2.4, Rust arm) | run.sh gives the reborn Borg its profile (default `harness/profiles/reborn-default.txt`: `name = value` lines in borg.txt's syntax) and its run directory. `config.txt` records the profile and its SHA-256. The Borg writes `params.json` (the parameter table: name, unit, range, default, value, evidence) and `trace.jsonl` (one line per think: turn, mode, goal and budget, the scored candidates of a contact think, choice, the mode rule, cooling modes and goal timing (`why`, #138), invariant violations, profile hash). The trace is written after each decision and never changes play. `tools/export-death.py` keeps `params.json` and the trace's last 200 lines (`trace-tail.jsonl`) in the dossier, with `profile_sha256` and `decision_trace` in `record.json`. CI checks both, in the run outputs and in the dossiers |
| `BORG_OBSERVATION_LOG=FILE`, `BORG_WORLD_LOG=FILE`, `BORG_KNOWLEDGE_LOG=FILE` (T2.1–T2.3, Rust arm) | The reborn Borg appends its `Observation` of every think to the first file, one JSON line each (a few hundred KB per think, so for short runs), and its world model's digest after that think to the second. It writes the game's static data (`Knowledge`, one JSON line) to the third at its first think. All are off by default, and recording does not change play. CI records a 3,000-turn run and checks: every observation round-trips; replaying the observations without the game rebuilds the live world after every think; the knowledge matches the game data; and the blows computed from it equal the engine's on every think (`BORG_OBSERVATION_LOG_CHECK=… BORG_WORLD_LOG_CHECK=… BORG_KNOWLEDGE_LOG_CHECK=… cargo test recorded`) |
| `suite.sh` `run_exit` (M1 re-review) | Writes `RUN/run_exit`, run.sh's exit status, for every run. |
| `check-characters.py` (E review, finding 1) | Run by `suite.sh`: each run played the race and class its suite line asks for (the names at those indices in `p_race.txt` and `class.txt`). Every suite line has the `menu` flag; without it, the birth screen could leave another character. A mismatch fails the suite. |
| `build.sh` `BORG_TEST_FEATURES` (M1 review) | Test-only cargo features for `BORG_IMPL=rust`, built into `build/rust-test/` so `build/rust/` is never a test build; `BUILD_ID` records them. `test-hallucinate` makes perception run while the player hallucinates and asserts on every think that the game's whole RNG is unchanged and that hallucination changes nothing observed. `test-perception` (T2.5) checks each think's observation and world against the engine through other routes and writes `RUN/truth.jsonl`; `tools/check-perception.py` reads it. CI builds both together and runs smoke and full with them. |

## References are regenerated only on purpose

A change that alters the Borg's keys makes the references differ. A PR that
means to change play says so and regenerates them with `make-reference.sh`. A
PR that does not must reproduce them.

## Run-derived P1 character tier

Build dependencies include OpenSSL's crypto headers/library (`libssl-dev` on
Debian/Ubuntu). The binary intentionally links libcrypto even for ordinary runs:
the engine attests the exact record bytes it applies with the maintained SHA-256
implementation, independently of the harness hash. Adapter/zoo hosts need the
matching runtime library. A custom in-tree cryptographic implementation is not
introduced in this milestone. `harness/capabilities.json` declares supported contracts; it is
bound to the exact checkout used by the suite adapter, not a moving branch.

`-- -start-build FILE` with `start-depth=N` reads a bounded declarative native
record (`borg-reborn-rs/character-build@1`, at most 16 KiB). The harness copies
it into the private run directory and hashes it; the engine independently hashes
and validates the bytes it applies. Summary/config use
`borg-reborn-rs/synthetic-start@2`, `start_build=record` and
`start_build_sha256`. Both birth and record starts use @2. The earlier @1 checkpoint is superseded: it did not prove the cave transition had completed before activation.

Pure casters additionally support `character-build@2`, with complete native
spell state and observed integral HP/SP (including HP=0 and exhausted mana),
rather than @1's resource refill. See [the exact contract](../docs/CHARACTER-BUILDS.md)
for fields, normalization, unsupported items and the pending Cemetery adapter.
Initial resource, armor and spell-count receipts describe actual application.

The P1 native adapter is intentionally restricted to Human Warriors, levels
1..15, and mundane base-kind items with their observed quantities, combat
bonuses, charges/fuel, equipped slots and kind awareness. It rejects artifacts,
egos, curses, brands/slays, nonzero item modifiers, changed kind flags/elements,
invalid slots/counts, inconsistent experience and HP progression, or a native
HP/SP calculation that differs from the declared record. It never drops an
unsupported item or silently substitutes a birth kit. No save-format payload
or private Borg state is accepted.

The record contains the source's natural stats, full rolled HP progression,
level/experience, gold, maximum HP/SP and physical kit. It preserves observed
combat-rune knowledge and carried-kind awareness; other historical knowledge is
not restored. The synthetic character starts at full HP/SP without timed effects
and with a fresh Borg. This is a declared tier projection, not continuation or
an exact copy of the source game's state. The engine records initial level and
maximum HP/SP so the suite can check the actual application.

Capture a tier from ordinary measured play with:

```sh
RUN_TIMEOUT_SECONDS=30 harness/run.sh /tmp/tier-source 101 0 0 20000 menu \
  -- -capture-build /tmp/warrior-level5.build -capture-build-level 5
```

Capture runs only at the first logged key at the requested level. It does not
advance play or load a save. Seed-101 level-5 capture/control streams demonstrate
RNG neutrality for that supported kit; this is not an exhaustive proof for every
kind. For every requested capture the summary records `build_capture_status`
(`captured` or `absent`) and `build_capture_requested_level`. Successful captures
also record turn/level and the engine-computed SHA256. A level not reached at a
pre-key boundary, including one skipped by multiple level gains, reports `absent`
and no digest. The game may still exit successfully: capture absence is separate
from its gameplay outcome. Existing capture files are never overwritten; unsupported
native properties fail explicitly and terminate the capture run as a harness
failure; such a run must not be used as a successful paired control. Normal run/death policy still applies to
this **ordinary source run**. The cemetery catalog builder verifies its clean
source/binary/process identity and capture digest before retaining provenance.
A one-observation P1 tier must not be described as a population-average kit.

Real checks are in `engine/tests/test_character_build.py`: capture on/off log
parity, independent file digest, observed tier application, repeated determinism,
invalid/unsupported requests and the absence of natural synthetic dossiers.

The milestone startup completes `on_leave_level`, native generation and
`on_new_level` under a harness-only wrapper. It verifies both `cave->depth`
and the absence of a pending generation, before activation and at the first
key. Both cores are covered by real repeated runs. The vendored core permits
only these two birth-option exceptions when the validated frontend authorizes
them; all other startup checks remain. Source @1 results are retained only as
superseded development evidence. New reports include generic refusal-message
counts and native progress; a refused Recall scroll is not evidence of a
qualified near-death or a survival improvement. One-way synthetic saves cannot
be loaded as unmarked ordinary games.

One-way starts retain the existing Borg decision policy. A hungry character
without food may select return-home actions that native rules refuse and may
stall; the six P1 cells do not exercise that state. Cap survival is not proof
that this policy handles long-horizon one-way play. #386's rule-aware policy
changes and starvation fixture are a separate follow-up, not part of this
startup/build contract.
