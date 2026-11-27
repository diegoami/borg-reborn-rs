# borg-reborn-rs: specification

*In effect since the owner merged the review PR #66 (2026-11-09). It is
the top-level contract of the repository: README, RULES, GOALS,
ARCHITECTURE and PLAN follow it, and where they disagree it wins until they
are brought into line (§9). Amended the same day: borg-zoo does every
comparison, and this repository delivers only our Borg (§6, §11).*

## 1. What it is

borg-reborn-rs is an automatic player (a Borg) for **Angband 4.2.6**,
upstream commit `574b618c3bfec595f3503a58f8e488b1c4ae3a46`, written in Rust
and designed from the ground up in how it plays. **borg-zoo measures it and
compares it with other Borgs**: the comparisons, the controls and the
statistics are borg-zoo's (owner, 2026-11-09). This repository delivers one
Borg, ours, through the zoo contract (§4).

It is **independent**: it owns every part it builds on, namely the
integration with the game, the seam, the bindings, the headless frontend,
the build, the harness, the evidence tools and the Borg itself. No part
depends on, follows, or is limited by any other Borg project, borg-rs in
particular, with one exception made on purpose: borg-rs's decision core,
vendored at a pin as `borg-core/` and selected by `BORG_CORE=borg-rs`
(§2 amendment, 2026-11-14); since #606 it is also the core a run with no
`BORG_CORE` plays (updated 2026-11-21, #816).

## 2. Independence

> **Amended 2026-11-14 (owner: the repivot, option C; docs/archive/ERRANDS.md, the errands retired 2026-11-27, #1339).**
> One part follows borg-rs on purpose, and is re-synced at each borg-rs
> milestone the owner picks. **The decision core:** borg-rs's crate is
> vendored at a pin as `borg-core/` and selected by `BORG_CORE=borg-rs`.
> Our changes to it are one patch: `src/shell.rs` (the errands) and the
> hooks marked `borg-reborn-rs (#207)` and `(#208)`. *(Updated 2026-11-21,
> #816: the core is the default since #606, and the patch has grown with
> track P and the errands; `scripts/borg-core-vendor.py` lists it, our own
> files as `OURS`, `PIN`, `ladders.txt` and eight modules, and the borg-rs
> files carrying our hooks as `HOOKED`, 53 of them. Updated 2026-11-27,
> #1339: the errands, `shell.rs`, `ladders.txt` and the plans were retired;
> the configuration readers moved to `config_file.rs`.)* The engine is
> upstream `574b618c` (#724; `42cd156ac`, the commit borg-rs v12 plays, until
> 2026-11-20, #215); borg-rs's one
> fork correction (`cmd-wizard.c`, debug mode) is not carried. §2.1's "no
> dependency" and §2.2's "no copy kept in sync" hold for everything else,
> and our own core still needs nothing from borg-rs.

1. **No dependency.** No build, test, check or decision needs a borg-rs
   checkout, a borg-rs pin, a borg-rs reference output or a borg-rs result.
   Nothing here must stay equal to anything in borg-rs. There is no re-sync
   duty.
2. **Copy, then own.** We may copy a file or a passage from borg-rs, or from
   any project the owner allows, when it is good. Copied code is ours from
   then on: we change it or delete it as we need. Each copy is recorded once,
   in `docs/PROVENANCE.md` (source, commit, path, licence: GPL-2.0, and what
   we changed). No copy is kept "in sync".
3. **References stay references.** Other projects' documents, specifications
   and results may be read and cited like any source (docs/SOURCES.md). The C
   Borg specification (`docs/c-borg/BORG_SPEC.md`), the strategy manual
   (S14) and the death registries remain inputs to design. A citation is not
   a dependency.
4. **Truth is the game.** Where a check needs ground truth (perception,
   determinism, the run contract), it comes from the engine and its data
   files, never from another Borg's state or outputs.

## 3. The game and its integration (the "engine")

### 3.1 The game

- Upstream Angband at `574b618c` (re-pinned from `42cd156ac` on 2026-11-20, #724, owner: "update engine"; `42cd156ac` was the pin from 2026-11-14, #215, the commit borg-rs v12 plays), vendored **byte for byte** as
  `engine/angband/`, next to the patch series and our build (TE.1). A test
  checks that the tree equals upstream's tree at that commit.
- The game rules are never changed. The only differences from upstream are
  the patches of §3.3, each one a harness hook that is inert unless a
  harness build turns it on.
- The source tree includes upstream's native Borg in `src/borg/`. The
  combined build compiles it alongside PBorg and applies the shared
  command prompt patch to the build copy (§3.6).

### 3.2 The seam

The combined build shares Angband's Borg command and key hook:

| Seam | Upstream | Our use |
| --- | --- | --- |
| Build switch | CMake `SUPPORT_BORG`, define `ALLOW_BORG` | on in every build |
| Entry | Ctrl-Z calls `do_cmd_try_borg()`, then `do_cmd_borg()` | Ctrl-Z opens the selected Borg prompt. The shared command handler dispatches to `do_cmd_pborg` or `do_cmd_cborg_key`. |
| Keys | `inkey_hack` (`ui-input.c`), read before every key | The active Borg installs its hook. |
| Reads | the game's globals and functions (`player`, `cave`, `map_info()`, `r_info`, …) | through generated bindings (§3.4), only the reads of the D3 list (§5) |

### 3.3 Harness hooks (our patch series)

Headless, measured runs need a few things the game does not offer. Each
is a small patch in `engine/patches/`, guarded by `#ifdef BORG_HARNESS` and
documented with its purpose and the evidence that it does not change play
when off. The full patch inventory is in [ENGINE.md](ENGINE.md).

| Hook | Where | Why |
| --- | --- | --- |
| Frontend registration | `main.c` (the module table), `main.h` (`init_borgh`, `help_borgh`) | upstream registers every frontend mode there; ours is mode `borgh` (§3.5) |
| Per-turn callback | `game-world.c` main loop | the exact turn boundary: stop at engine turn T itself |
| Seeding | `z-rand.c` | a run's seed fixes its game |
| Fixed date | `mon-make.c` | holiday monsters depend on the clock |
| Message log | `message.c` | every message, Borg notes included, to `messages.log` |
| Key and state logs | `ui-input.c`, `ui-game.c` | `keys.log` (the determinism oracle) and `state.log` |
| Borg selection | `cmd-misc.c`, `borg/borg-commands.c` | Share Ctrl-Z and its warning. Default to PBorg and switch Borgs with `B`. |

The patch series is applied at build time to a copy of the pristine tree
(§3.6), so `engine/angband/` stays upstream's bytes. borg-rs's equivalents may be copied as a
starting point. borg-rs's other engine changes are **not** carried: its
recording hooks, its wizard cure-all correction (reached only by cheat
death, which D5 excludes) and its adopted C Borg carries.

### 3.4 Bindings

The Rust bindings to the engine are **generated by us**: bindgen run on the
pinned headers, limited to the symbols of the D3 list (§5). They are
checked in as `borg-reborn-rs/src/bindings.rs` with the script and the
version that made them, and a test regenerates them and compares. The
c2rust headers copied from borg-rs are retired.

### 3.5 The headless frontend

Our own frontend, `engine/frontend/main-borgh.c` (outside the pristine
tree, compiled with it, registered as mode `borgh`), runs a game without a terminal. Its
command line and outputs are a **fixed contract**, because borg-zoo calls
them (§4, `docs/ZOO-INTERFACE.md`). The names below are today's, and they
stay.

- **Invocation:**

  `angband -mborgh [-n] -uBorg -duser=… -dsave=… -darchive=… -dpanic=…
  -dscores=… -- OPTIONS`

- **Options:**
  - `-seed N`, `-race I`, `-class I`, `-menu-letters` (pick race and class
    by the birth menu's letters);
  - `-turns T` (cap) and `-exact-turns` (stop at engine turn T itself);
  - `-depth D`, `-keys K` (caps);
  - `-load FILE` (a living save; without `-n`) and `-keep-load-rng`;
  - `-keylog`, `-msglog`, `-statelog` and `-summary FILE`;
  - `-screen` (a screen dump on an unexpected end).
- **Outputs**, per run directory:
  - `keys.log`, `messages.log` and `state.log`;
  - `summary.json`, with at least: `seed`, `race`, `race_idx`, `class`,
    `class_idx`, `stop_reason`, `game_turns`, `player_turns`, `depth`,
    `max_depth`, `clevel`, `max_clevel`, `dead`, `death_cause`, `keys`,
    `exact_turns`, `loaded`, and `effective_settings` (the C Borgs'
    settings);
  - `process.json`: the supervised child's outcome;
  - for the Rust Borg, `trace.jsonl` and `params.json` (§7).
- **Stops:** death, the turn cap (exact or after the key), a stall (no
  decision for N seconds), a wall-time limit, a key limit.
- **Determinism:** the same build, seed, options, configuration and
  environment (§4) give byte-identical `keys.log`, `messages.log` and
  `state.log`.

borg-rs's `borgh` (1,247 lines) is the starting point to copy from. We
keep what this contract needs and drop the rest (recording, frames, native
replay). A test checks that the frontend answers every flag above and writes
every field.

### 3.6 Build

- **One build system:** `engine/CMakeLists.txt` builds a patched copy of
  `engine/angband/`, the frontends and both Borgs.
- **The source list matches upstream:** `engine/sources.cmake` copies all
  209 upstream core sources, including the 59 native Borg sources. The
  build also links PBorg's Rust static library. `engine/tests/test_build.py`
  checks the C source list against upstream.
- **Entry point:** `harness/build.sh` with `BORG_IMPL=<impl>` builds
  `build/<impl>/game/angband` and writes `build/<impl>/BUILD_ID`. Both the
  name and the layout are part of the zoo contract (§4).
- **Combined build (#1288):** `BORG_IMPL=rust` links the Rust staticlib and
  the native C Borg into one executable. Ctrl-Z opens PBorg by default.
  The prompt points to `?`, which lists `B` for switching between PBorg
  and CBorg for the session (#1301). The harness retains `BORG_IMPL=rust`
  and the `build/rust/` layout for the Zoo.
- **Identity:** `BUILD_ID` holds the hashes of the build's inputs (engine
  tree, patches, frontend, Borg sources, Cargo.lock), the binary's
  `binary_sha256` and the implementation. A run records it.

## 4. The harness and the run contract

The harness is ours. Today's scripts (run, suite, diff, identity, supervise),
copied from borg-rs and adapted, are kept as our code and may change
freely, **except for the zoo contract**: the build entry point, the
frontend's flags and outputs, and `harness/borg.txt` (§3.5, §3.6, below).

- **Run contract** (GOALS D7):
  - the exact turn boundary;
  - the depth stream (`depths.json`);
  - each run's exit status (`run_exit`);
  - the suite manifest with binary hashes at start and end, and
    `result=OK|FAILED`;
  - every attempted run counts;
  - the **configuration** and the **environment** below.

  `check-boundary.py` (ours) is kept. The primary endpoint and every other
  statistic are computed by borg-zoo, so `tools/endpoint.py` goes (§8).
- **Configuration**, part of the run record:
  - **The Rust Borg's profile** (`harness/profiles/`, its hash in
    `config.txt`).
  - **The game options** the Rust Borg sets, decided in `docs/ENGINE.md`
    and written into `summary.json`. The C Borg sets its own in
    `borg_reinit_options` (`borg-init.c:403-444`: `auto_more` and
    `easy_alter` off, `pickup_always` on, …). The Rust Borg sets only
    `auto_more` **on** (`easy_alter` is not an option of 4.2.6; the Rust
    Borg set it until TE.5 and the engine ignored it, docs/ENGINE.md). Each
    difference from the C set is declared
    with its reason (spec §1.2 warns that the options change the screen the
    Borg reads).
  - **`harness/borg.txt`** is the PBorg settings template. The harness and
    Zoo apply the run overrides and convert it to `user/pborg.yaml` before
    launching PBorg. CBorg uses upstream's settings in `user/borg.txt`.
    The template descends from borg-rs's v12 file, with `borg_munchkin_by_class` and `borg_self_scum` set to
    `FALSE` (owner, 2026-11-14: no stair-scumming, no town scumming), and
    three keys of ours that select the decision core (`borg_reborn_core`),
    an errand (`borg_reborn_errand`) and the fear stop
    (`borg_reborn_stopscared`; the last two retired with the errands,
    #1339). Our own core still reads nothing else in
    it. *(Updated 2026-11-21, #816: it now also carries its own shipped
    values for borg-rs's settings, #479, the track P parameters as
    `borg_param_*` lines, and further keys of ours such as `borg_scary_list`
    and `borg_world`.)* Until 2026-11-14 it was inert here, and its `borg_self_scum = TRUE`
    (upstream's default) let the C Borgs money-scum in the zoo's
    `4.2.6-reborn-c-*` specimens; the zoo was told (RULES 7).
- **Environment:** `TZ=UTC`, `LANG=C.UTF-8` and `LC_ALL=C.UTF-8` for every
  run. The fixed-date hook sets the time `mon-make.c` reads to 0, but
  `localtime()` still turns it into a calendar date in the run's time zone
  (1 January 1970 at UTC, 31 December 1969 west of it). So the time zone is
  part of determinism.
- **Suites** (`harness/suites/`, kept): our own files of runs (name, seed,
  race, class, cap, flags): smoke and full, and their exact-boundary
  variants. Population suites (the class suites) are the zoo's (§6).
  Every line has the `menu` flag, and `suite.sh` checks that each run
  played the race and class it asks for (E review, finding 1). Seeds are chosen once and frozen.
- **References:** the Rust Borg's smoke and full outputs at a reviewed
  commit. They are regression references, not oracles; none comes from
  another project. Before today's references are dropped, the new
  integration must reproduce them (§10, E's exit).
- **Deaths:** every natural death of a measured run (`harness/run.sh`,
  the zoo's cells) gets a dossier (RULES 12); watched games are not
  recorded (owner, 2026-11-10).
  `tools/export-death.py` remains a pointer to the cemetery-owned implementation, with the trace tail and the
  parameters (T2.4).
  - `docs/deaths/` then holds two generations: D0001–D0525 on today's
    borg-rs-based tree, each dossier keeping its own `source_sha256`, and
    the runs after E as new encounter families. The canonical registry is
    **[borg-cemetery](https://github.com/diegoami/borg-cemetery)**; this
    repository writes and reconciles it through `BORG_DEATH_RECORDS_DIR`
    (`docs/deaths/POINTER.md`).
  - `tools/reconcile-deaths.py` keys on (family, key stream), so the two
    never mix.
- **The zoo:** borg-zoo measures our Borg through **the zoo contract**,
  written down in `docs/ZOO-INTERFACE.md`:
  - `harness/build.sh`, `build/<impl>/game/angband` and
    `build/<impl>/BUILD_ID` (with `binary_sha256`);
  - the PBorg settings template, converter and YAML declaration in `BUILD_ID` as specified in `docs/ZOO-INTERFACE.md`;
  - the frontend's mode, flags and outputs (§3.5).

  This is the interface the zoo's existing "borg-rs" adapter calls. Keeping
  it is an interface we choose, not a dependency. A test runs the zoo's
  adapter at its pin against our build (today's `harness/zoo-check.py`). If
  the zoo later adds an adapter of its own for us, the contract moves with
  it by agreement (RULES 7).

## 5. The D3 list (information level)

The Borg sees what the C Borg sees, and no more (GOALS D3, spec §1.1). This
section summarises; **`docs/ENGINE-READS.md` is the normative list**: every
symbol the bindings expose and the boundary module uses, each with its
justification. A test fails when the boundary uses a symbol not on it.

- **Channel A, the screen:**
  - the message line (row 0), with the attribute of its first cell and the
    cursor position. The C Borg reads both to tell a prompt from a message
    and to catch `-more-` (`borg-messages.c:183-238`), and the Rust Borg will
    need them to answer prompts;
  - the store header (row 3, columns 1–4).
- **Channel B, direct reads:** own status, gear, spells, the map panel
  through `map_info()` with the cave predicates the C Borg uses, the real
  race and condition of monsters seen, the real kind of objects seen, every
  store's stock, and the static data (`r_info`, `k_info`, `e_info`,
  `a_info`, classes, races, stores, the energy table, the stat tables).
- **Declared extras:** engine values the C Borg computes for itself, read
  so that our own computations can be checked against the engine, or
  because the engine's figure is the correct one:
  - `pack_slots_used()`: the engine's pack count, quiver included
    (REDESIGN-LESSONS L8);
  - `state.stat_ind` and `state.num_blows` (checks of T2.3);
  - `known_state.ac + known_state.to_a` (the AC the player is shown);
  - `opts.opt[birth_percent_damage]` (a birth option the blows formula
    needs);
  - `price_item()`: a ware's price as the shop shows it (the C Borg
    computes its own, `borg_price_item`).

  Each is in `docs/ENGINE-READS.md` with its reason.
- **Writes** are limited to the key hook, the game options (§4) and the RNG
  swap.

## 6. Controls and comparisons: borg-zoo's

borg-zoo chooses the Borgs it compares ours with, runs them, and computes
every endpoint and statistic (owner, 2026-11-09). The combined build ships
upstream CBorg as an interactive option (#1288). The Zoo owns the control arms:

- The Zoo builds and runs its own control specimens of upstream CBorg and
  better-borg-c milestones. Same-seed pairing with them is the
  zoo's concern: for example, it can build a C Borg on our engine and
  frontend, since both are open and documented (`docs/ENGINE.md`,
  `docs/ZOO-INTERFACE.md`).
- This repository never requests control runs (owner, 2026-11-11). The
  zoo runs the original C Borg and better-borg-c in its own experiments;
  a run we ask for is of our Borg only, on suites and seeds the controls'
  results already cover.
- What this repository owes the zoo is the zoo contract (§4): a Borg that
  builds and runs under it, deterministic, with its run record and its
  death evidence.
- Today's arms `c`, `c-pristine` and `c-bbc-m7`, their overlays and
  references, and #56 (`c-bbc-m8`) are dropped. better-borg-c's milestone 8
  is borg-zoo#36.
- borg-rs's Borg is not compared here either. Its zoo results, like all zoo
  results, are context for our design.

## 7. The Borg

Unchanged in intent from docs/ARCHITECTURE.md, re-founded on §3–§5:

- **L0 Engine boundary:** `engine.rs`, the only `unsafe` code, over our
  bindings.
- **L1 Perception:** the `Observation`, one plain serialisable snapshot per
  think (T2.1).
- **L2 World model:** map memory, monster tracks, store memory and the
  character model, changed only by `World::update` (T2.2).
- **L3 Knowledge:** the static data read from the engine, plus lifted
  formulas with citations and tests against the engine's values (T2.3).
- **L4–L6** (assessment, strategy, executor): as ARCHITECTURE.md, from the
  next milestones.
- **Cross-cutting:** the parameter table and profiles, and the decision
  trace (T2.4). Determinism: the Borg's own RNG, with the game's RNG
  restored around every engine read on every path (a drop guard).
- **Perception checks:** the engine is the truth (owner, 2026-11-09). Each
  think's observation and world model are compared with engine facts
  reached through other routes than the observation's own reads: the
  player's map memory (`player->cave`), the visibility of every monster,
  the pack count, and the frontend's `state.log`. Each difference is fixed
  or declared with a spec citation.

## 8. What happens to what exists

| Part | Today | Decision | Why |
| --- | --- | --- | --- |
| `angband/` | borg-rs's tree: upstream plus its hooks, recording frontends, a gameplay correction and its v8 C Borg carries (39 files of `src/borg/` differ from upstream) | **Replace** with upstream `e473720` byte for byte | §3.1; borg-rs's carries and corrections are not ours |
| borg-rs's harness hooks | in `angband/` | **Re-found** as our patch series (copy as a start) | §3.3 |
| `main-borgh.c` | borg-rs's headless frontend, in the tree | **Re-found** as `frontend/main-borgh.c`, same mode and contract | §3.5 |
| Recording frontends (`native-replay.*`, frames) | borg-rs's recording toolkit | **Drop** | not needed to play or measure; can come back as our own later |
| `borg-reborn-rs/src/headers/` | c2rust bindings copied from borg-rs | **Replace** with our generated bindings | §3.4 |
| `borg-rs/` shim package, its `Cargo.toml` workspace member | exists so borg-rs's CMake finds a crate of that name | **Drop** | §3.6: our CMake builds our crate |
| `Cargo.toml`, `Cargo.lock` | the workspace | **Keep**, without the shim member | |
| `harness/build.sh` | borg-rs's build with our overlays | **Re-found** on §3.6, same name and layout | the zoo calls it |
| `harness/measurement/`, `control-overlay.py` | borg-rs-specific build helpers and the C controls' overlays | **Drop** | §3.6, §6 |
| `run.sh`, `suite.sh`, `diff`, `identity.sh`, `supervise.py` | copied from borg-rs, adapted | **Keep** as ours, adapt to the new frontend | §4 |
| `harness/borg.txt` | Settings inherited from borg-rs | PBorg template for the harness and Zoo. Both generate `user/pborg.yaml` for play. | §4 |
| `harness/suites/`, `harness/profiles/` | ours | **Keep** | §4 |
| `check-boundary.py`, `check-extra-load.sh`, `zoo-check.py`; `tools/depth-stream.py`, `export-death.py`, `reconcile-deaths.py`, `tools/tests/` | ours | **Keep** | §4 |
| `tools/endpoint.py` | ours: the primary endpoint over local outputs and zoo cells | **Drop** | the zoo computes every statistic (§6) |
| `harness/reference/smoke-c`, `smoke-pristine`, `full-pristine`, `smoke-c-bbc-m7` | references of the C arms | **Drop** | §6 |
| Arms `c`, `c-pristine`, `c-bbc-m7`; #56 (`c-bbc-m8`) | Historical C Borg control arms | Zoo-owned. The combined release ships upstream CBorg (#1288). | §6 |
| The copied `tools/` (GUI recording, replay viewer, human takeover, native recording, the death workflow, their tests, 2 C files with 2 headers) and the root launchers | copied from borg-rs and better-borg-c in T1.6 | **Dropped** in TE.6, all of it (owner, §11 q1): the death tooling imports the recording toolkit | replaced by `tools/rerun-death.py` (rerun a death with the Borg's logs), `tools/death_registry.py` (the index) and `watch-borg.sh` (one launcher) |
| `tools/IMPORTED.json`, `tools/README.md` | the import record of T1.6 | **Replace** with `docs/PROVENANCE.md` | §2.2 |
| `engine.rs`, `borg.rs`, `observation.rs`, `world.rs`, `knowledge.rs`, `params.rs`, `trace.rs` | ours (M1, T2.1–T2.4) | **Keep**, re-based on the new bindings | the Borg layers are ours |
| `docs/deaths/` (D0001–D0525) | ours, on borg-rs's tree | **Keep** as history, a generation of its own; the canonical registry now lives in [borg-cemetery](https://github.com/diegoami/borg-cemetery) | §4 |
| `pins.json`: borg-rs | build source, `engine_tree` record, `borg_dir` | **Reference only**: `engine_tree` and borg-rs's `borg_dir` removed; the refresh report's watch of cited C Borg functions uses the angband pin's `src/borg` | §2 |
| `reference/borg-rs/` | copies of borg-rs docs | **Keep** as reference | §2.3 |
| `docs/c-borg/`, `docs/strategy/`, `reference/deaths/`, `reference/zoo/`, `docs/research/` | sources and digests | **Keep** | |
| S12 (`reference/strategy/`) | draft manual | **Retired** in TE.8 (owner, 2026-11-09) | S14 supersedes it |
| `docs/ZOO-CHANGE-REQUEST.md` | the request done by borg-zoo#32 | **Superseded** by `docs/ZOO-INTERFACE.md`; kept as history | §4 |
| borg-zoo specimens `4.2.6-reborn-*` | at `2dd11fd`, through the zoo's borg-rs adapter | **Re-specify** `4.2.6-reborn-rust` after the new integration; the `-c-*` specimens are the zoo's to keep or retire | §4, §6 |
| `.github/workflows/ci.yml` | CI on today's tree | **Rewrite** in E (§10) | |

## 9. Documents and rules that change

- **README and AGENTS.md:** drop "with the conventions, structure,
  interfaces, tooling and external behaviour of borg-rs"; state §1–§2.
- **RULES 9** ("the engine tree is upstream with the hooks, corrections and
  carries inherited from borg-rs") becomes: the engine tree is upstream's
  bytes, plus our declared patch series.
- **RULES 22:** drop the borg-rs re-sync.
- **GOALS:**
  - D2 (engine: "the `angband/` tree of borg-rs") becomes upstream plus our
    patches;
  - D4 ("the substrate may reuse borg-rs code") becomes §2.2: copy, then
    own;
  - D7: the primary endpoint is computed by borg-zoo, not by
    `tools/endpoint.py`;
  - D8 ("same-frontend controls built here"): the controls and every
    comparison are borg-zoo's (§6), and borg-rs's roles go.
- **docs/SOURCES.md:** borg-rs's row becomes "reference, copied from
  where useful".
- **harness/README.md:** rewritten for the new build and frontend.
- **New documents:**
  - `docs/ENGINE.md` (§3, the option set of §4), replacing
    `docs/ENGINE-TREE.md`;
  - `docs/PROVENANCE.md` (§2.2), replacing `tools/IMPORTED.json`;
  - `docs/ENGINE-READS.md` (§5);
  - `docs/ZOO-INTERFACE.md` (§4).
- **PLAN.md** is re-planned (§10).

## 10. Plan

The milestone method stays: review PRs, a reviewer of another model family,
stop after each milestone. M0 and M1 stay as history. M2's tasks T2.1–T2.4
are done and kept; T2.5 and the milestones after it are re-planned:

| Milestone | Delivers | Exit |
| --- | --- | --- |
| **E. Our own integration** | §3 and §4: pristine `angband/`, patch series, our frontend, generated bindings, our CMake, the Rust Borg built on them, the harness and run contract on it, the configuration of §4, new references, the zoo contract written down and checked, CI; the removals of §8; the documents of §9 | see below |
| **M2 (rest). Perception, checked** | T2.5 against the engine (§7), the D3 list and its test (§5); T2.1–T2.4 re-checked on the new integration | the M2 exit of PLAN.md, with the engine as truth |
| **M3–M7** | as PLAN.md: a Borg that plays; danger and survival; readiness; the family; deeper | unchanged |

**E's exit** (all must hold):

1. `angband/` equals upstream `e473720`, and the Rust Borg builds from
   it, our patches and our frontend only.
2. **Reproduction before replacement.** The new build reproduces today's
   Rust smoke and full key streams. Any difference is explained and
   recorded before the old references are replaced (RULES 22). Then the
   option set of §4 is settled, and new references are made with their
   configuration hashes.
3. A seed run twice gives identical logs.
4. The run contract holds on smoke-exact.
5. Every natural death of a measured run has a dossier.
6. The zoo's adapter, at its pin, builds and runs our Borg (§4).
7. **CI proves:**
   - the tree equals upstream;
   - the patches apply;
   - the source-list copy matches upstream's;
   - the frontend contract (§3.5);
   - determinism;
   - the run contract;
   - the D3 list test (§5);
   - the death reconciliation.
8. No build, test or check reads borg-rs.

E comes first, and nothing else starts before it is reviewed: everything
after it runs on it.

## 11. Decisions on the open questions (owner, 2026-11-09)

1. **Tools:** first, keep the death tooling and drop the rest. Then
   (TE.6), since the death tooling imports borg-rs's whole recording
   toolkit: drop all copied tools and write one lean tool of ours instead.
   `tools/rerun-death.py` reruns a death from its build and seed with the
   Borg's logs and checkpoint saves. One launcher, `watch-borg.sh`, stays to
   watch the Borg play.
2. **The zoo:** the interface its current adapter calls is kept and
   written down as the zoo contract (§4, `docs/ZOO-INTERFACE.md`).
3. **Controls:** borg-zoo does every comparison, and this repository
   delivers only our Borg (§6). `c-bbc-m8` is the zoo's (borg-zoo#36); #56
   is closed.
4. **Engine changes of other Borgs:** the question goes with the controls.
   For the record: better-borg-c's milestones 7 and 8 change nothing in
   upstream's `src/` and `lib/` outside `src/borg/`.
