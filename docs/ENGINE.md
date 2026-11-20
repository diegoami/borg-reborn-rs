# The engine (`engine/`)

*docs/SPEC.md §3. Task TE.1 (#71) makes the tree and the patch series; the
frontend (TE.2), bindings (TE.3) and build (TE.4) follow.*

## The tree: `engine/angband/`

Upstream Angband at `574b618c3bfec595f3503a58f8e488b1c4ae3a46`,
**byte for byte** (`engine/UPSTREAM`). It was extracted from the upstream
repository (`git archive` of that commit), not from borg-rs. Its tree id,
`88b243a5eba440cef82007fd5a2c6477467a95f7`, equals the commit's tree.
`engine/tests/test_engine.py` checks that at `HEAD` and refuses local
changes. The tree is never edited: changes are patches applied to a copy at
build time (`engine/apply-patches.sh DEST`).

Re-pinned from `42cd156ac` on 2026-11-20 (#724, owner: "update engine"):
`574b618c` is upstream `master` 10 commits later. The changes that touch the
game: trap display made consistent (`cave-map.c`, `ui-target.c`,
`ui-display.c`, `trap.c`, `trap.h`), two effect handlers
(`effect-handler-attack.c`, `effect-handler-general.c`), `datafile.c` and
`obj-init.c`. Three commits are the C Borg's (`src/borg`, not built here; the
fixes are carried into borg-core and listed in `engine/upstream-borg.txt`);
the rest are CI, Windows and SDL2. The patch series applied unchanged (two
hunks at new offsets: 02, 5 lines; 09, 2 lines), the core source list is
unchanged, and the bindings are unchanged (`trap.h` adds a function they do
not bind). Every game plays as before: smoke-borg-rs 3/3, smoke-rust 3/3,
smoke-exact-rust 3/3, full-rust 45/45 identical, so no reference is
re-recorded (each `REFERENCE` still names the engine it was made on).

The previous re-pin: re-pinned from `e473720` on 2026-11-14 (#215, owner: every Borg plays
upstream Angband): `42cd156ac` is upstream `master` 12 commits later, the
commit borg-rs v12 plays. The changes that touch play: timed effects with
`COMPLETE_REST` (`player_timed.txt`, `player-timed.c`) decide when a complete
rest ends; deep descent flushes the command queue; a rest's continuation is
a background command; `TERM_XTRA_CLEAR` is split from `total_erase`. The
patch series applied unchanged (two hunks at new offsets), the core source
list is unchanged, and one bindings constant moved (`TMD_FLAG_NONSTACKING`,
1 to 2, unused).

On the new engine our own Borg plays 44 of the full suite's 45 games as
before; smoke and smoke-exact are unchanged. The one that moves is the
druid of seed 1024 (`full-2-druid-4`): it splits after a rest at game turn
3,521, and where it died to a kobold at turn 41,864 it now lives to the
cap. The full reference is regenerated for it. borg-rs's core
(`BORG_CORE=borg-rs`, no errand) plays all 45 exactly as a fresh build of
borg-rs v12.

One file, `lib/user/save/Makefile`, is ignored by upstream's own
`.gitignore` but tracked upstream, so it is force-added here too.

borg-rs's tree (`angband/` at the root) was used until TE.4 switched the
build to `engine/`, and removed in TE.6.

## The patch series: `engine/patches/`

Each patch says why it exists, and each guard is `BORG_HARNESS` (the
harness hooks) or `USE_BORGH` (the frontend mode). With both undefined,
every patched file is upstream's code. There is one declared exception, and
the test checks the rest:

| Patch | Files | What |
| --- | --- | --- |
| `01-frontend-registration` | `main.c`, `main.h` | registers mode `borgh` in upstream's module table and declares `init_borgh`, `help_borgh` |
| `02-turn-callback` | `game-world.c` | `borgh_turn_hook()` after every game turn: the exact turn boundary |
| `03-seeding` | `z-rand.c` | `Rand_init()` seeds from `borgh_seed` when the harness sets one |
| `04-fixed-date` | `mon-make.c` | the time holiday monsters read is 0 when seeded |
| `05-message-log` | `message.c` | `borgh_message_hook()` sees every `message_add()`, repeats included |
| `06-key-log` | `ui-input.c` | `borgh_key_hook()` sees every key `inkey_hack` returns |
| `07-command-boundary` | `ui-game.c` | `borgh_command_hook()` at each game-command boundary |
| `08-town-queries` | `store.c/.h`, `obj-gear.c/.h` | M13 read-only exact-stack price and inventory-capacity queries, under `BORG_HARNESS` (#328) |
| `09-synthetic-start` | `game-world.c` | Harness-only wrapper around native leave/generate/arrival before synthetic Borg activation (#371); normal game loop and generation rules unchanged |

**The declared exception:** `04-fixed-date` splits
`struct tm *date = localtime(&cur_time);` into a declaration and an
assignment, so that the guarded line can go between them. With the guards
off, the code is the same.

The hook variables and the header the patches include (`main-borgh.h`)
belong to our frontend (TE.2). The hooks come from borg-rs's tree
(docs/PROVENANCE.md). Its other changes to the engine are not carried: the
recording hooks (`BORG_RECORDING` in 8 files), the wizard cure-all
correction (`cmd-wizard.c`, reachable only by cheat death, which D5
excludes), its CMake changes, its recording frontends, and its C Borg
carries in `src/borg/`.

## The frontend: `engine/frontend/`

`main-borgh.c` and `main-borgh.h`, mode `borgh` (docs/SPEC.md §3.5; task
TE.2). They are compiled with the patched tree; the hook variables of the
patch series are declared in `main-borgh.h`. The command line and the
`summary.json` fields are the zoo's contract, and
`engine/tests/test_frontend.py` checks them in the source, together with a
compile against the patched tree. TE.4 adds the same checks on the binary.
The options for borg-rs's recording, fixtures, rune stop and cheat_live are
not carried (docs/PROVENANCE.md).

## The bindings: `borg-reborn-rs/src/bindings.rs`

Generated by bindgen 0.73.2 from the pristine headers
(`engine/bindings/generate.sh`, `wrapper.h`; docs/SPEC.md §3.4). The
allowlist names the functions, globals and constants of
`docs/ENGINE-READS.md`, and bindgen adds the types they need. Its
compile-time layout assertions (66) check every struct's size and field
offsets against clang's view of the C headers. `generate.sh --check`
regenerates the file and compares; CI runs it. One symbol, `inkey_hack`, is
declared in no upstream header, so `engine.rs` declares it itself. The c2rust
headers copied from borg-rs are gone.

`engine/tests/test_reads.py` is the D3 audit:
- every symbol `engine.rs` uses is in `docs/ENGINE-READS.md`, and every
  listed one is used;
- only `engine.rs` touches the bindings.

## The build: `engine/CMakeLists.txt`

Our CMake project (docs/SPEC.md §3.6; task TE.4):
- it compiles a patched copy of the pristine tree (`build/<impl>/tree/`,
  remade by `engine/apply-patches.sh` whenever a patch changes), our
  frontend, and the Rust Borg's staticlib (`cargo build -p borg-reborn-rs
  --features borgh`), into `game/angband`, with the data in `game/lib`;
- it does not build the C Borg (`src/borg/`): the Rust Borg exports
  `do_cmd_borg` in its place;
- the source list is a copy of upstream's (`engine/sources.cmake`, 150 of
  upstream's 209 core sources: all but the 59 of `src/borg/`), and
  `engine/tests/test_build.py` checks that it is upstream's list minus the C
  Borg;
- the flags are upstream's: C99 with GNU extensions, `-O3` in Release, the
  warnings, `ALLOW_BORG`, the self-contained data paths `./lib/`, plus
  `BORG_HARNESS` and `USE_BORGH` for our hooks and frontend;
- the version is upstream's own: no `BUILD_ID` from our checkout's git state,
  so the binary does not depend on the checkout;
- `-DSUPPORT_X11=ON` also builds the X11 frontend (for the launcher, TE.6).

`harness/build.sh` (`BORG_IMPL=rust`, the zoo's entry point) runs it and
writes `build/rust/BUILD_ID`. The build inputs are the paths `harness/identity.sh` lists: `engine/`
(tests aside), `borg-reborn-rs/`, `borg-core/` (borg-rs's vendored core,
since #214), `Cargo.toml`, `Cargo.lock`, and the recorder's inputs
(`engine/recorder-hooks.patch`, `recorder-pin.json`,
`vendor/angband-recorder`, since #211).

**Reproduction (TE.4):** the Rust Borg built here plays byte-identical keys,
messages and states to its build on borg-rs's tree: smoke 3/3, full 45/45,
smoke-exact 3/3, and the hallucination test smoke 3/3.

borg-rs's tree (`angband/`), the `borg-rs/` shim crate and
`harness/measurement/` were kept until TE.6 for the copied death tooling,
and removed with it.

## Determinism

A run is fixed by four things (docs/SPEC.md §4):
- the seed;
- the fixed date;
- the environment. `harness/run.sh` runs the game with `TZ=UTC`,
  `LANG=C.UTF-8` and `LC_ALL=C.UTF-8`, because `localtime()` turns the
  fixed date into a calendar date in the run's time zone;
- the configuration below.

## The game options

The Rust Borg sets one game option when it starts (`GAME_OPTIONS` in
`borg.rs`), and an option the engine does not know stops it. The frontend
writes every option in force at the stop into `summary.json`
(`game_options`), so each run records them.

| Option | Rust Borg | C Borg (`borg_reinit_options`, `borg-init.c:403-444`) | Why we differ |
| --- | --- | --- | --- |
| `auto_more` | **on** | off | The C Borg turns `-more-` off to read each batch of messages behind it. The Rust Borg's executor (T3.1) keeps `auto_more` on: the prompt catcher answers a `-more-` should one appear, and the game never stalls on one, while the observation still reads the message line at each think. The cost, declared: a batch that scrolls by between thinks is not parsed from the screen (the harness's `messages.log` keeps every message for the dossier). T3.3 revisits this if the strategy needs the C Borg's full message stream. |
| `rogue_like_commands`, `pickup_always`, `pickup_inven`, `pickup_detail`, `use_old_target`, `show_piles`, `show_labels`, `show_flavors`, `hitpoint_warn` | not set (the game's defaults) | set | The Rust Borg's keys do not depend on them yet. The executor (T3.1) leaves each as the game's default; the pickup options belong to T3.4, when the Borg first picks things up, and the rest to the tasks that first use them. |

**Not options in 4.2.6:** `easy_alter` and `easy_open`. The Rust Borg set
`easy_alter` until TE.5, and the engine ignored it without a word
(`option_set` returns false). The C Borg's calls for both are leftovers too.
Walking into a closed door opens it in 4.2.6 anyway (`move_player` calls
`do_cmd_alter_aux`).

spec §1.2 warns that the options change the screen the Borg reads. Each
difference from the C set is declared here, with its reason.

## References

`harness/reference/smoke-rust/` and `full-rust/` hold the reborn core's
(`BORG_CORE=reborn`) smoke and full outputs (keys, messages and state,
xz-compressed, plus `summary.json`) at the commit recorded in each
`REFERENCE` file; `smoke-borg-rs/` holds the default (borg-rs) core's smoke
(#606). They are regression references, not oracles (docs/SPEC.md §4). CI
compares the reborn core's smoke with `smoke-rust` and the default core's
smoke with `smoke-borg-rs`. Full is compared locally, under the lock, when the Borg's code
changes.

TE.5 made them from the build on our own integration, after TE.4's
reproduction check: that build plays the same keys as the build on
borg-rs's tree, smoke and full.

**Regenerated after E's review** (finding 1): the full suite's lines had no
`menu` flag, so 20 of its 45 runs played another race or class than they
asked for. With the flag, those 20 runs play their requested characters,
and their keys, messages and states change; the other 25, smoke and
smoke-exact are unchanged. Each `REFERENCE` now takes its provenance from
the suite's manifest (finding 8).

P3 (#244) adds an optional diagnostic directory default in the borgh
frontend: when BORG_RUN_DIR is absent and -summary names a file, its parent
is used for parameter/decision artifacts. Explicit caller paths win. The
profile contract tests the pinned zoo adapter with no policy environment;
no game/RNG read or upstream engine edit is involved.
