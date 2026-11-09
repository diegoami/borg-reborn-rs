# borg-reborn-rs: specification

*Written 2026-11-09 at the owner's request. The owner accepted the draft
and sent it to review with the work since `m1`. It takes effect when the
owner merges that review PR (`main` → `reviewed`). Until then nothing below
is implemented. From then on it is the top-level contract of the
repository: README, RULES, GOALS, ARCHITECTURE and PLAN follow it, and
where they disagree it wins until they are brought into line (§9).*

## 1. What it is

borg-reborn-rs is an automatic player (a Borg) for **Angband 4.2.6**,
upstream commit `e473720ed88a0f0c7ca66815cef3000eb4a38bc3`, written in Rust
and designed from the ground up in how it plays. Its results are compared
with control C Borgs on the same seeds, under the same run contract, through
our own harness and borg-zoo.

It is **independent**: it owns every part it builds on, namely the
integration with the game, the seam, the bindings, the headless frontend,
the build, the harness, the evidence tools and the Borg itself. No part
depends on, follows, or is limited by any other Borg project, borg-rs in
particular.

## 2. Independence

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
   Borg specification (`reference/c-borg/BORG_SPEC.md`), the strategy manual
   (S14) and the death registries remain inputs to design. A citation is not
   a dependency.
4. **Truth is the game.** Where a check needs ground truth (perception,
   determinism, the run contract), it comes from the engine and its data
   files, never from another Borg's state or outputs.

## 3. The game and its integration (the "engine")

### 3.1 The game

- Upstream Angband at `e473720`, vendored **byte for byte** as `angband/`.
  A test checks that the tree equals upstream's tree at that commit.
- The game rules are never changed. The only differences from upstream are
  the patches of §3.3, each one a harness hook that is inert unless a
  harness build turns it on.
- Upstream's own Borg (`src/borg/`, 118 files) stays in the tree unmodified.
  It is the original C Borg, the control `c-pristine` (§6).

### 3.2 The seam

Upstream already has the whole Borg seam, and we use it as it is:

| Seam | Upstream | Our use |
| --- | --- | --- |
| Build switch | CMake `SUPPORT_BORG`, define `ALLOW_BORG` | on in every build |
| Entry | `^Z` → `do_cmd_try_borg()` (`cmd-misc.c:115`) → `do_cmd_borg()` | the Rust Borg exports `do_cmd_borg`; the C Borgs provide their own |
| Keys | `inkey_hack` (`ui-input.c:130`), read before every key | the Rust Borg installs its hook; the C Borgs install theirs |
| Reads | the game's globals and functions (`player`, `cave`, `map_info()`, `r_info`, …) | through generated bindings (§3.4), only the reads of the D3 list (§5) |

The Borg implementation is chosen at build time (§3.6). Exactly one of the
Rust Borg or a C Borg's `src/borg/` is linked.

### 3.3 Harness hooks (our patch series)

Headless, measured runs need a few things the game does not offer. Each
is a small patch in `engine/patches/`, guarded by `#ifdef BORG_HARNESS` and
documented with its purpose and the evidence that it does not change play
when off. In today's tree the hooks proper are about 33 changed lines in 6
files, plus 14 in `main.c` and `main.h` for the frontend's registration.
The rest of borg-rs's changes to those files are its recording includes,
which are not carried.

| Hook | Where | Why |
| --- | --- | --- |
| Frontend registration | `main.c` (the module table), `main.h` (`init_borgh`, `help_borgh`) | upstream registers every frontend mode there; ours is mode `borgh` (§3.5) |
| Per-turn callback | `game-world.c` main loop | the exact turn boundary: stop at engine turn T itself |
| Seeding | `z-rand.c` | a run's seed fixes its game |
| Fixed date | `mon-make.c` | holiday monsters depend on the clock |
| Message log | `message.c` | every message, Borg notes included, to `messages.log` |
| Key and state logs | `ui-input.c`, `ui-game.c` | `keys.log` (the determinism oracle) and `state.log` |

The patch series is applied at build time to the pristine tree (§3.6), so
`angband/` stays upstream's bytes. borg-rs's equivalents may be copied as a
starting point. borg-rs's other engine changes are **not** carried: its
recording hooks, its wizard cure-all correction (reached only by cheat
death, which D5 excludes) and its adopted C Borg carries.

### 3.4 Bindings

The Rust bindings to the engine are **generated by us**: bindgen run on the
pinned headers, limited to the symbols of the D3 list (§5). They are
checked in under `borg-reborn-rs/src/bindings/` with the script and the
version that made them, and a test regenerates them and compares. The
c2rust headers copied from borg-rs are retired.

### 3.5 The headless frontend

Our own frontend, `frontend/main-borgh.c` (outside `angband/`, compiled
with it, registered as mode `borgh`), runs a game without a terminal. Its
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

- **One build system:** a CMake project of ours (`engine/CMakeLists.txt`)
  that compiles the pristine `angband/` sources, our patches, our frontend
  and exactly one Borg implementation.
- **The source list is ours:** upstream's `CMakeLists.txt` lists every
  `src/borg/*.c` inside its core library, and the Rust Borg exports
  `do_cmd_borg` itself, so upstream's file cannot be used as it stands
  (the symbol would be defined twice). Ours carries a copy of upstream's
  source list, recorded in `docs/PROVENANCE.md`. The `rust` arm leaves out
  `src/borg/*.c`. A test compares the copy with upstream's list, so a
  difference is caught.
- **Entry point:** `harness/build.sh` with `BORG_IMPL=<impl>` builds
  `build/<impl>/game/angband` and writes `build/<impl>/BUILD_ID`. Both the
  name and the layout are part of the zoo contract (§4).
- **Implementations** (`BORG_IMPL`):
  - `rust`: the Rust staticlib, built by cargo;
  - `c-pristine`: upstream `src/borg/` as is;
  - `c-bbc-mN`: a better-borg-c milestone tag's `src/borg/`, pinned by
    commit.
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

  `check-boundary.py` and `tools/endpoint.py` (ours) are kept.
- **Configuration**, the same for every arm and part of the run record:
  - **The C Borgs' `harness/borg.txt`** (kept, ours, its hash in
    `config.txt`). Every exploit GOALS D5 excludes is off:
    - `borg_munchkin_start = FALSE`;
    - `borg_munchkin_by_class = FALSE` (milestone 8 on);
    - `borg_lunal_mode = FALSE`, `borg_self_lunal = FALSE`;
    - `borg_money_scum_amount = 0`, `borg_self_scum = FALSE`;
    - `borg_cheat_death = FALSE`.

    Today's file has **`borg_self_scum = TRUE`** (upstream's default too).
    That lets a C control money-scum for a ware from character level 10
    (`borg-store-buy.c:141-153`), which D5 excludes. It is corrected in E.
  - **The game options** every arm runs with are decided in one place
    (`docs/ENGINE.md`). The C Borgs set theirs in `borg_reinit_options`
    (`borg-init.c:403-444`: `auto_more` and `easy_alter` off,
    `pickup_always` on, …). The Rust Borg sets `auto_more` and `easy_alter`
    **on**. In E either the Rust Borg adopts the C set, or each difference is
    declared with its reason. The options in force go into `summary.json`.
  - **The Rust Borg's profile** (`harness/profiles/`, its hash in
    `config.txt`).
- **Environment:** `TZ=UTC`, `LANG=C.UTF-8` and `LC_ALL=C.UTF-8` for every
  run. The fixed-date hook sets the time `mon-make.c` reads to 0, but
  `localtime()` still turns it into a calendar date in the run's time zone
  (1 January 1970 at UTC, 31 December 1969 west of it). So the time zone is
  part of determinism.
- **Suites** (`harness/suites/`, kept): our own files of runs (name, seed,
  race, class, cap, flags): smoke, full, the class suites, and their
  exact-boundary variants. Seeds are chosen once and frozen.
- **References:** each arm's smoke and full outputs at a reviewed commit.
  They are regression references, not oracles; none comes from another
  project. Before today's references are dropped, the new integration must
  reproduce them (§10, E's exit).
- **Deaths:** every natural death gets a dossier (RULES 12).
  `tools/export-death.py` (ours) is kept, with the trace tail and the
  parameters (T2.4).
  - `docs/deaths/` then holds two generations: D0001–D0525 on today's
    borg-rs-based tree, each dossier keeping its own `source_sha256`, and
    the runs after E as new encounter families.
  - `tools/reconcile-deaths.py` keys on (family, key stream), so the two
    never mix.
- **The zoo:** borg-zoo measures our arms through **the zoo contract**,
  written down in `docs/ZOO-INTERFACE.md`:
  - `harness/build.sh`, `build/<impl>/game/angband` and
    `build/<impl>/BUILD_ID` (with `binary_sha256`);
  - `harness/borg.txt`, which the zoo copies;
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
    needs).

  Each is in `docs/ENGINE-READS.md` with its reason.
- **Writes** are limited to the key hook, the game options (§4) and the RNG
  swap.

## 6. The controls

Every comparison runs our Rust Borg against **control C Borgs built by
the same integration**, on the same tree, frontend, patches, configuration,
seeds and contract (GOALS D8):

- **`c-pristine`** is upstream's own `src/borg/`: the original C Borg, its
  defects included (better-borg-c's milestones exist to fix them). A test
  checks that its sources are upstream's bytes. That proves **provenance,
  not correctness**: it is a control, not an oracle.
  - Today's tree is different: its `src/borg/` is borg-rs's adopted C Borg,
    39 files changed from upstream, and `c-pristine` is built by overlaying
    upstream's sources on it. On the pristine tree, no overlay is needed.
- **`c-bbc-mN`** is a better-borg-c milestone tag's `src/borg/`:
  - the overlay equals the tag's blobs, which a test checks;
  - better-borg-c's engine-side changes up to that milestone are
    inventoried before the arm reports any result (§11 q4);
  - the configuration of §4 applies, `borg_munchkin_by_class = FALSE`
    included from milestone 8 on (owner, #56).
- A new better-borg-c milestone is a new arm; an existing arm is never
  redefined (RULES 22).
- borg-rs's Borg is **not** a control. borg-zoo's results for it are
  context, like any other zoo result.

The tree's own C Borg is upstream's, so today's arm `c` (borg-rs's adopted
C Borg) disappears.

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
| `harness/measurement/`, `control-overlay.py` | borg-rs-specific overlay helpers | **Replace** with §3.6 | |
| `run.sh`, `suite.sh`, `diff`, `identity.sh`, `supervise.py` | copied from borg-rs, adapted | **Keep** as ours, adapt to the new frontend | §4 |
| `harness/borg.txt` | the C Borgs' configuration, from borg-rs | **Keep**, ours, corrected to §4 (`borg_self_scum = FALSE`) | zoo contract; D5 |
| `harness/suites/`, `harness/profiles/` | ours | **Keep** | §4 |
| `check-boundary.py`, `check-extra-load.sh`, `zoo-check.py`; `tools/depth-stream.py`, `endpoint.py`, `export-death.py`, `reconcile-deaths.py`, `tools/tests/` | ours | **Keep** | §4 |
| `harness/reference/smoke-c`, `smoke-pristine`, `full-pristine` | outputs of borg-rs's builds | **Drop after** the new integration reproduces them (§10) | §4 |
| `harness/reference/smoke-c-bbc-m7` | ours | **Regenerate** on the new integration, after the same check | |
| Arm `c` | borg-rs's adopted C Borg | **Drop** | §6 |
| The rest of `tools/` (25 Python files, 8 of them tests, and 2 C files with 2 headers) and the 7 root launchers | copied from borg-rs and better-borg-c in T1.6 | **Drop**, except what §11 q1 keeps and one launcher to watch the Borg play (Ctrl-Z, z) | most of it is borg-rs's recording format |
| `tools/IMPORTED.json`, `tools/README.md` | the import record of T1.6 | **Replace** with `docs/PROVENANCE.md` | §2.2 |
| `engine.rs`, `borg.rs`, `observation.rs`, `world.rs`, `knowledge.rs`, `params.rs`, `trace.rs` | ours (M1, T2.1–T2.4) | **Keep**, re-based on the new bindings | the Borg layers are ours |
| `docs/deaths/` (D0001–D0525) | ours, on borg-rs's tree | **Keep** as history, a generation of its own | §4 |
| `pins.json`: borg-rs | build source, `engine_tree` record, `borg_dir` | **Reference only**: `engine_tree` and borg-rs's `borg_dir` removed; the refresh report's watch of cited C Borg functions uses the angband pin's `src/borg` | §2 |
| `reference/borg-rs/` | copies of borg-rs docs | **Keep** as reference | §2.3 |
| `reference/c-borg/`, `reference/c-manual/`, `reference/deaths/`, `reference/zoo/`, `docs/research/` | sources and digests | **Keep** | |
| S12 (`reference/strategy/`) | draft manual | **Retire** (owner, 2026-11-09) | S14 supersedes it |
| `docs/ZOO-CHANGE-REQUEST.md` | the request done by borg-zoo#32 | **Superseded** by `docs/ZOO-INTERFACE.md`; kept as history | §4 |
| borg-zoo specimens `4.2.6-reborn-*` | at `2dd11fd`, through the zoo's borg-rs adapter | **Re-specify** after the new integration | §4 |
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
  - D8: borg-rs's roles go, and the oracle for `c-pristine` (the zoo's
    `4.2.6-rs-m0-c`) becomes the reproduction check of §10.
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
| **E. Our own integration** | §3 and §4: pristine `angband/`, patch series, our frontend, generated bindings, our CMake, the arms (rust, c-pristine, c-bbc-m7, and c-bbc-m8 if §11 q3 says so), the harness and run contract on it, the configuration of §4, new references, CI; the `tools/` and arm-`c` removals; the documents of §9 | see below |
| **M2 (rest). Perception, checked** | T2.5 against the engine (§7), the D3 list and its test (§5); T2.1–T2.4 re-checked on the new integration | the M2 exit of PLAN.md, with the engine as truth |
| **M3–M7** | as PLAN.md: a Borg that plays; danger and survival; readiness; the family; deeper | unchanged |

**E's exit** (all must hold):

1. `angband/` equals upstream `e473720`, and every arm builds from it, our
   patches, our frontend and its own Borg sources only.
2. **Reproduction before replacement.** With today's configuration, the
   new `c-pristine` reproduces today's `c-pristine` smoke and full key
   streams, and the new `rust` arm reproduces today's Rust smoke and full
   key streams. Any difference is explained and recorded before an old
   reference is replaced (RULES 22). Then the configuration of §4 is
   applied (`borg_self_scum = FALSE`, the option set), and new references
   are made with their configuration hashes.
3. A seed run twice gives identical logs on every arm.
4. The run contract holds on smoke-exact for every arm.
5. Every natural death has a dossier.
6. The zoo's adapter, at its pin, builds and runs a specimen of each arm
   (§4).
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

## 11. Open questions for approval

The reviewer of #66 (MiniMax-M3) answered each question; its answers are
given as recommendations. The owner decides.

1. **Tools:** drop all of `tools/` except one launcher (§8), or keep some?
   *Recommendation:*
   - keep the death tooling as ours: `record-death.py`,
     `death_workflow.py`, `borg_deaths.py`, `borg_saves.py` and their tests.
     M4's calibration and fixtures from deaths need to re-read recorded runs,
     and `export-death.py`'s dossiers point to that tooling;
   - drop the GUI, replay viewer, human takeover and native recording.
2. **The zoo:** keep the interface the zoo's current adapter calls, or ask
   for a new adapter at once? *Recommendation:* keep it, written down as the
   zoo contract (§4). Asking now would leave the project without a
   yardstick in M3–M5.
3. **c-bbc-m8** (#56): build it in milestone E, or keep it in M4 as filed?
   *Recommendation:* build it in E as an arm, since E pins the arms'
   configuration anyway. E's exit must not depend on it.
4. **better-borg-c's engine-side changes** in `c-bbc-mN`: carry them as
   declared patches, or run every arm on the identical engine?
   *Recommendation:* run every arm on the identical engine (upstream plus
   our patch series). Then, case by case:
   - a change to the game rules is not carried; it is declared as a
     difference;
   - a fix for a crash or undefined behaviour that would make a control
     fail for reasons unrelated to play is carried as a declared patch;
   - the inventory is a precondition for the arm's first result.
