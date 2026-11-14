# Handover: extract the recorder into `angband-recorder`

**Date:** 2026-11-14 · **For:** a local session with four repositories as subdirectories:

```
workspace/
├── borg-rs/            diegoami/borg-rs          (the Rust Borg; most of the recorder work happened here)
├── better-borg-c/      diegoami/better-borg-c    (the C Borg; where the recorder was born)
├── borg-reborn-rs/     diegoami/borg-reborn-rs   (the new bot; recordings will be its training data)
├── angband-recorder/   diegoami/angband-recorder (new, empty: the owner creates it on GitHub)
└── borg-cemetery/      diegoami/borg-cemetery    (the runs: death records now, recordings later)
```

---

## 1. The decision (owner, 2026-11-14)

The recorder is a change to the Angband engine. Vanilla Angband has nothing like it. It moves into its own project, **`angband-recorder`**, which is developed there and **pinned** from its consumers. Each consumer keeps its own patched `angband/` tree, with its own fixes and Borg, and vendors one tagged recorder release into it.

Further owner decisions, the same day:
- **A clear separation between Angband and the recorder.** Angband is the host and exposes a small fixed interface; the recorder is a plug-in.
- **The recorder only in the Rust builds.** The consumers are **borg-rs and borg-reborn-rs**. The recorder is **removed from better-borg-c**, so a C-only build is not a constraint. better-borg-c is still in the workspace: to compare its copy, then to remove it there.
- **The recorder is written in Rust** (§2a).
- **Branching is interactive, inside the game** (§2b; borg-rs #63 holds the same design).
- **The runs are kept in `diegoami/borg-cemetery`**, not in the code repositories (§2c).

The longer-term goals it serves (borg-rs issue #62 and its sub-issues #63–#69):

1. record a game, played by a human, a Borg or both;
2. replay it, and move back and forth in it;
3. from any point, e.g. a few turns before a death, branch and play on, saving the branch **in the same file**; then branch from branches;
4. later, a site where people upload and download recordings;
5. later still, a dataset of recorded games to train `borg-reborn-rs`. The existing Borg was built for 3.0.x and does not win 4.2.6; learning from recorded play is the favoured approach.

## 2. What `angband-recorder` does (the proposed scope)

**In it:**

1. **The engine patch** (C, compiled in only with a build option, off by default):
   - **Records** everything the game reads from outside the program:
     - every keyboard/mouse poll, **including empty polls that time out** (lazy movement depends on them), with wait/take mode and key-queue head/tail;
     - every `time()` call (the date is read on every monster generation, for holiday monsters);
     - the PID that seeds the RNG;
     - the bytes of every file at the moment it is read (`.prf` keymaps, `borg.txt`, ...);
     - the terminal size.
   - **Replays** by feeding those back. It never supplies the game's decisions or its RNG, so an identical replay proves the recording is complete.
   - **Records a bot's keys** at the engine's own bot hook (`inkey_hack`, under `ALLOW_BORG`). Any bot that drives the game through that hook is covered: the C Borg, the Rust Borg, possibly borg-reborn-rs.
   - **Writes per-frame observations** for viewing and checking only: a screen delta, HP/SP/depth/gold, the full RNG state, and a world fingerprint (a hash of a save written by a forked child).
2. **One recording format** (borg-rs #63): one file holding a recording and its tree of branches, plus a written spec.
3. **The tools:** `record`, `replay --verify`, `view` (seek, inventory, spells, combat), `branch --from <segment>@<frame>`, `list`.
4. **Its own round-trip test** on unmodified vanilla Angband plus the patch, independent of any Borg.

**Not in it** (stays in the consumers):
- each Borg and its logic;
- borg-rs's C/Rust parity harness (`angband/src/main-borgh.c`, `harness/`), which compares Borgs, not recordings;
- the death workflow, the zoo, the named-character launchers;
- each fork's engine bug fixes.

**How consumers pin it:**
- `angband-recorder` publishes tagged releases: the source files, a hook patch against a **stated vanilla Angband version**, and the CMake snippet.
- Each consumer vendors one release and records the pin (tag, commit, file hashes), as borg-rs's `tools/IMPORTED.json` does today.
- A check fails the build if the vendored copy differs from the pin.
- Upgrading means moving the pin and applying the new release.

## 2a. Language: Rust, with thin C hooks

| Part | Language | Note |
|---|---|---|
| The engine hooks (about 21 call sites in 10 files) | C, a few lines each | they sit in Angband's C code and call the recorder through one header |
| The host interface for the recorder's menu | C | drawing on the terminal and reading a key (Angband's own `Term_*` calls), one command that opens the menu, and the start-up entry point (§2b) |
| The recorder core: capture, replay, the streams, the clock wrapper (`__wrap_time` as a Rust `extern "C"` function) | Rust | |
| Per-frame observations (RNG state, screen, HP/depth) | a thin C callback that hands over the numbers | borg-rs's Rust engine structures (`borg-rs/src/headers/`) are tied to one engine version; a C function is the stabler interface |
| The file format with its tree, the menu, scrubbing, verify, branch, list | Rust | one implementation of the format for everything |
| A library to read recordings (borg-reborn-rs's training data; later the browser viewer via WebAssembly for the site) | Rust crate | |

**The linking catch:** a C program cannot link two Rust static libraries, because both carry Rust's runtime and the symbols clash. So the recorder is a **crate**, linked through the consumer's existing Rust library: borg-rs's Rust Borg, and borg-reborn-rs's, if it links into the engine; find out (§5 step 1). The C hooks are compiled in only with the recorder's build option, which the Rust builds set. borg-rs's C-Borg oracle build and its parity harness keep it off.

## 2b. Branching inside the game (owner, 2026-11-14)

**What the owner wants:** an option inside Angband to load a recording, pick a turn, play from there, save, go back to the choice point, and choose between the two paths, or branch again. It must be interactive, in the game window. Today's `--play-from-turn` is one level, from the command line, with the branch in a separate directory and no way back.

**The model: a tree of nodes in one file**

```
node = { id, parent, branch point (move number in the parent),
         start save, the inputs recorded after it,
         save points (move -> save), a label, who played it (human / Borg) }
```

- **The root** is the original game: its start save plus its inputs.
- **A branch's start save** is the save point it was branched from. **Replaying a branch** loads that save and feeds its inputs; it never needs its parents. That save-then-new-process path is the one verified byte for byte (§4).
- **The container:** e.g. zip, with `manifest.json` (version, build identity, the nodes) and one directory per node.
- **A move** is a command boundary: the recorder already writes a frame at each command (`record_frame("command", …)`).

**Rolling saves (the owner's point, 2026-11-14): no waiting.** While a node is recorded, the recorder takes **save points** at command boundaries, each in a forked copy of the game so the live game is undisturbed (the checkpoints' mechanism: `-checkpoint-every`, `docs/checkpoints.md`):
- **every move** in a rolling window of the last few hundred moves (a setting): where a player rewinds before a death;
- **every K turns** elsewhere. Today's headless recordings already write one every 1,000 turns.

"Play from here" is offered at save points, and restarts the game directly on that save: **no replay, no Borg re-run**. Between save points the menu snaps to the nearest earlier one. Replaying forward to an in-between move is not reliable: the save's monster list is compacted, so monster order and the RNG can differ, and a re-run Borg has no memory. For branching the save point is exact: the branch starts from it, has a new future, and replays exactly from it. Size: a save is a few hundred KB, so the window is tens of MB raw; compression is to be measured (#67).

**What the player sees:**
1. A recordings menu: a new command, or an entry on the start-up screen. It loads a file and shows the tree.
2. Pick a node and scrub its moves. Each move's screen is rebuilt from the recorded frames, as `--view-only` does today, with the turn, depth, HP and messages. **No restart and no wait**, Borg parts included. Save points are marked.
3. "Play from here" at a save point:
   1. a branch being played is closed into the file (inputs, save points, final save);
   2. the game process replaces itself (`exec`) with a fresh start on that save, recording into a new child node from its first instant. The window closes and reopens once;
   3. the player has the keyboard (the Borg off unless turned on; then it starts with no memory).
4. Save or quit, or open the menu: the branch is stored, and the menu returns at the choice point, both paths visible.
5. Choose: scrub either path, play from any save point of any node, or keep playing.

**The separation: Angband provides only**
- the input hooks (exist): terminal polls, `time()`, the seeding PID, file reads, the bot's keys (`inkey_hack`);
- the observation hooks (exist): frames, messages;
- a save-point hook: a forked save at a command boundary (as the checkpoints do);
- drawing and key reading for the recorder's menu;
- one command that opens that menu;
- a start-up entry point: "start from save S, recording into node X of file F".

**Everything else is the recorder's:** the format, the tree, the save-point schedule, the menu, scrubbing, verification, and starting the restart. Angband never loads a second save in a running process: its globals are not built for it.

## 2c. Where the runs are kept: borg-cemetery

- **Now:** `deaths/` holds D0001–D0006 and the registry, moved from borg-rs (byte-identical to borg-rs `4ed3263` `docs/deaths/`; borg-cemetery commit `d168adf`). New deaths are written there by pointing borg-rs's launchers at a checkout: `BORG_DEATH_RECORDS_DIR=/path/to/borg-cemetery/deaths`. Its README has the layout and rules.
- **Recordings: decided (owner, 2026-11-14): slim recordings committed to borg-cemetery, and release assets only if necessary.** A full bundle today is 100–180 MB (the frozen executable about 14 MB; the checkpoint saves and frames most of the rest), and GitHub refuses files over 100 MB. So **the recorder's file format is designed to be small** (borg-rs #67):
  - it holds only the input streams, the start save, the save points (compressed or as deltas) and a small digest per frame; the screens and logs are rebuilt on replay and on scrubbing;
  - it names the build it needs, and does not carry the executable (#66);
  - the target is a few MB for a 20,000-turn game.

  A recording still too large goes into a borg-cemetery GitHub release, with an index entry under `recordings/` pointing at it.

## 3. What exists today, in borg-rs (read this first)

The overview of every recording/replay feature is **`borg-rs/docs/RECORDING.md`**. It is on branch `feature/recordings-with-branches`, not on `main` yet. Its "Starting a recording from a save" section is the analysis behind §4.

### The engine side (on `main`, since commit `75fbcc6`, 2026-11-07, part of the accepted M5 tag `m5`)

| File | What |
|---|---|
| `angband/src/native-replay.c` (392 lines), `native-replay.h` | the recorder |
| `angband/src/replay-native-inventory.h`, `replay-native-spells.h` | observation views |
| `angband/CMakeLists.txt` | `option(SUPPORT_BORG_RECORDING ... OFF)` → `-DBORG_RECORDING`, adds `native-replay.c`, links `-Wl,--wrap=time`. Linux only (fork + linker wrapping) |
| `main.c` | `record_init()` at start (must be on from process start); an exit frame |
| `ui-term.c` | `Term_inkey` wrapped: replay feeds events, else the real `recording_real_inkey`; every result recorded |
| `ui-input.c` | `record_borg_key()` on the `inkey_hack` path (the bot's keys) |
| `ui-game.c` | frames at each command and tick |
| `message.c` | `record_message()` |
| `z-file.c` | `record_read_path()` in `file_open`: read-time file capture and redirection |
| `z-rand.c` | `#define getpid() record_pid()` |
| `savefile.c` | skips the char dump when in the snapshot child |
| `main-test.c` | the test front end's geometry/env for replay (`BORG_REPLAY_WIDTH/HEIGHT/KEY_SIZE/...`) |

- **Size:** about 21 `#ifdef BORG_RECORDING` blocks in 10 files. Every hook is behind the flag, so the default build has none of it.
- **Runtime env:** `BORG_RECORD_DIR` (record), `BORG_REPLAY_DIR` (+ `BORG_REPLAY_SYS`, `BORG_REPLAY_*` geometry) (replay).
- **Recorded streams:**
  - inputs: `events.txt`, `clock.txt`, `pid.txt`, `file-inputs.txt` + `file-input-NNNNNN.bin`;
  - the bot's keys: `actions.txt`;
  - observations: `native-messages.jsonl`, `geometry.json`, `frames.jsonl`;
  - the footer: `record-complete.json`. If any stream is left unconsumed on replay, the footer is not written and verification fails.
- **A separate thing:** `BORG_HARNESS` (`main-borgh.c`, hooks in 9 files: fixed seed, pinned holiday date, key/message/state logs) is borg-rs's parity harness, **not** the recorder. It does not move.

### The tool side (Python, about 3,000 lines)

- `scripts/replay-borg.py` (`--verify-only`, `--view-only`, `--from-turn`, `--play-from-turn`), `scripts/play-from-replay.py`, `scripts/play-branch.py`.
- `tools/`: `borg_runtime.py`, `borg_gui.py`, `toolkit_native.py`, `toolkit_common.py`, `human_takeover.py`, `borg_replay_viewer.py`, `replay_observations.py`, `borg_saves.py`, `start_recorded_borg.py`, `run_report.py`, `display_font.py`, `native-combat.h`.
- Bundles: `borg-rs-headless-recording@1`, `borg-rs-gui-recording@1`, with the `borg-rs-toolkit-overlay@1` extension. A bundle freezes the executable, data, config, origin save and seed.
- The scripts moved from the top level to `scripts/` on `feature/recordings-with-branches`. On `main` they are still at the top level.
- These tools mix generic recorder work (verify, view, branch) with borg-rs specifics (Rust build identity, the death workflow, named characters). Separating them is part of the extraction.

### Provenance (from `borg-rs/tools/IMPORTED.json`)

| borg-rs file | from better-borg-c |
|---|---|
| `angband/src/native-replay.c`/`.h`, its snapshot helper | `08a5cd019` `tools/native-replay.c`, `.h`, `tools/native-observer.c` (merged there in PR #147) |
| `tools/borg_replay_viewer.py`, `replay-native-inventory.h`, `replay-native-spells.h`, `replay_observations.py` | `85b6ba8e` |
| `tools/native-combat.h`, `tools/toolkit_native.py`, `scripts/play-from-replay.py` | `a4084032` |
| `scripts/replay_next_death.py`, `produce_death.py` | `22f817c9` |

**borg-rs's additions over the donor:**
- the engine wiring under `angband/src/` with the CMake option (the donor has it under `tools/`; how it is wired into better-borg-c's game is **not known yet**: check);
- the clock wrapper also covering the Rust staticlib's calls;
- the Rust Borg's keys;
- borg-rs's own bundle schemas.

Original licence and credit are preserved. Angband's licence is in `angband/docs/copying.rst` (GPL-2.0-or-later or the Angband licence).

## 4. Already established (keep these)

- **The save holds the RNG.** A 4.2 save restores the RNG state (`savefile.c` block `"rng"` → `rd_randomizer`), overwriting the start-up seed. The headless front end keeps it with `-keep-load-rng`.
- **Recording from a save works.** It was checked on 2026-11-14 in borg-rs:
  1. a Warrior from birth for 20,000 turns;
  2. the same character continued from that save in a **new process** for 20,000 more;
  3. `replay-borg.py --verify-only` on run 2 gave **VERIFIED**: keys, messages, player state, terminal frames, 18 scheduled saves and the final save, byte for byte.

  It was one headless Borg run, not a human GUI run (borg-rs #68).
- **Saving changes the running game.** The saver compacts the monster list in the live process (`wr_dungeon` → `compact_monsters`), and the running process holds state no save has: the target, the repeat state, the screen, the Borg's memory. So "start recording now" in a running game is unsafe. **The design is: switch on = save at a command boundary, then restart from that save with recording on from process start** (borg-rs #64).
- **A Borg on a branch restarts fresh.** The Borg's private memory is not in a save, so a Borg that continues a branch starts fresh there. A human continuing a branch needs nothing.
- **Alternative, parked:** record an *unmodified* vanilla binary by intercepting `time`, `getpid`, file reads and terminal input from outside (an `LD_PRELOAD` shim). It would be Linux-only, and it couldn't easily take per-frame RNG snapshots or bot keys.
- **Also parked (owner's idea):** porting the engine to Rust (about 164k lines of C). It would give cheap in-memory rewind, a training engine for borg-reborn-rs, and WASM for the site; the cost is exact parity with the C engine and porting every upstream fix by hand. Check first whether borg-reborn-rs already has or plans its own simulation.

## 5. The work, in order

1. **Compare the three copies** (better-borg-c's only to take what is better; it is then removed there):
   - borg-rs: `angband/src/native-replay.*`, the hooks above, `tools/`;
   - better-borg-c: `tools/native-replay.c` and whatever wires it in: hooks in its `angband` source? a build-time patch?
   - borg-reborn-rs: does it run the C engine? Does it link a Rust library into it (that decides how the recorder crate is linked, §2a)? Through `inkey_hack`? Does it record anything?

   Write down what each has that the others lack, and pick the base.
2. **Pick the engine base:** which vanilla Angband version the hook patch targets (4.2.5? the 4.2.6 the forks track? borg-rs just imported upstream fixes in v12R, PR #58, tag `4.2.6-impact-v12`).
3. **Set up `angband-recorder`:**
   - a README with §2's scope;
   - `src/` (the recorder), `patches/` (the hooks against the stated base), the CMake snippet;
   - `tools/` (generic verify/view/branch, borg-specific parts removed), `spec/` (the format);
   - `tests/` (the round trip on vanilla + patch);
   - a licence and credits (from the donor headers and `IMPORTED.json`);
   - tag `v0.1.0` when the round trip passes.
4. **Pin it from borg-rs first** (its copy is the most complete): vendor the release, link the crate through the Rust Borg's library, add the pin check, and keep the parity suites green (§7). Then borg-reborn-rs.
5. **Remove the recorder from better-borg-c**, in that repository, following its own `CLAUDE.md`. Keep the provenance: the borg-rs and angband-recorder files keep their "adapted from better-borg-c" credits.
6. **Then the recorder features**, in `angband-recorder`, in this order:
   - the tree file and in-game branching (#63, §2b);
   - switch on in game (#64; the same start-up entry point);
   - the in-game menu and scrubbing (#65 becomes part of it);
   - portability (#66), smaller files (#67), the GUI human check (#68). Those issues are in borg-rs now. Move or mirror them to `angband-recorder` (the owner decides; GitHub can transfer issues between repositories of the same owner).

## 6. Open questions for the owner

1. Which copy is the base, after the comparison. (Decided: the language is Rust, the consumers are the Rust builds only, and branching is in the game.)
2. Which vanilla Angband version the patch targets.
3. Vendored-with-pin (proposed) vs. a git submodule vs. a patch applied at build time. With a Rust crate, a git dependency pinned to a tag in the consumer's `Cargo.toml` covers the Rust part; the C hooks still need the vendored patch.
6. The key or menu entry that opens the recordings menu, and whether it is also on the start-up screen.
7. ~~How recordings are stored in borg-cemetery~~: decided, slim recordings in Git, release assets only if necessary (§2c).
4. Whether issues #62–#69 move to `angband-recorder`.
5. Whether the env var names stay `BORG_RECORD_DIR` / `BORG_REPLAY_DIR` (existing bundles and scripts use them) or get a neutral prefix, with the old names accepted.

## 7. Rules from the consumers (respect them when pinning)

- **borg-rs:**
  - the Rust Borg's keypress stream must stay byte-identical to the C Borg's for the `current` control. The recorder is off in the parity build (`harness/build.sh` does not set `SUPPORT_BORG_RECORDING`), so this should hold, but run the smoke suite to show it;
  - preserve historical tags, adoption records and provenance (`IMPORTED.json`);
  - no derived tags without the owner;
  - raw saves, recordings and runs stay outside Git;
  - **never** open issues, PRs or comments on `angband/angband`.
- **No benefit claims:** state what was run and what was not.
- **better-borg-c and borg-reborn-rs:** read their own `CLAUDE.md` before changing anything there.

## 8. Recipes (borg-rs)

```bash
cd borg-rs
BORG_IMPL=c    bash harness/build.sh
BORG_IMPL=rust bash harness/build.sh
cargo test --release                  # debug does not link
cargo clippy --release --all-targets -- -D warnings

# parity (the recorder off)
BORG_IMPL=c    bash harness/suite.sh smoke OUT-c
BORG_IMPL=rust bash harness/suite.sh smoke OUT-rust
python3 harness/diff --suite smoke --expect-impl rust OUT-rust OUT-c

# a recording and its verification (headless; keep the data out of .angband-rust)
export BORG_PLAY_DATA_DIR=/tmp/rec/data BORG_DEATH_RECORDS_DIR=/tmp/rec/deaths BORG_DEATH_WORKFLOW_DIR=/tmp/rec/wf
./scripts/run-borg-headless.py Test --new --seed 4242 --race 0 --class 0 --turns 20000
./scripts/run-borg-headless.py Test --turns 20000          # continues from the save
./scripts/replay-borg.py /tmp/rec/data/recordings/<bundle> --verify-only --output /tmp/rec/verify
```

(`scripts/` on `feature/recordings-with-branches`; on `main` the launchers are at the top level.)

## 9. Other threads (orientation, not this session's work)

- **Buying** (borg-rs epic #36, phase 4 in shadow mode, branch `feature/town-inventory-strategy`): its own session has an open question for the owner.
- **The trip policy** (borg-rs #60, draft PR #61, branch `feature/trip-policy`): a separate handover exists. It merges cleanly onto the strategy branch.
- **The town session tools** (PR #56, closed as merged into the strategy branch): the object log, the store's one-key buy (`store_buy_direct`, a C engine change, off by default, **a fork proposal only**, not in `main`), the end-of-session feedback. Frozen.
