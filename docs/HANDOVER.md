# Handover

*Where the work stands, for the next session. Update it in every PR.*

## 2026-11-10: M2 reviewed and tagged

Kimi K3.1 Pro's fourth pass on #99, at `7cec235`: **M2 done**, with no
findings left. The owner had #99 merged (`reviewed` at `4ea0bb0`); `m2`
is tagged there, and the GitHub milestone is closed. The four passes'
findings were all fixed in #100–#104, so none carry over to M3.

**Next:** M3, per PLAN.md, with S14 and S15 (borg-runner, #103, below)
as its game knowledge.

## 2026-11-10: a new source, borg-runner (S15)

The owner added [borg-runner](https://github.com/diegoami/borg-runner) as a
source of game knowledge: the accumulated knowledge of agent players.
Language-model agents play upstream 4.2.6 (`e473720`, our engine) from the
screen and record every key; what they learn goes into one page,
`manuals/INSTRUCTIONS.md`.
- **Pin:** `borg-runner` at `e829a01`. At that commit LEARNED.md and
  MANUAL-GAPS.md had just been folded into the page.
- **Copies** (`reference/borg-runner/`):
  - `manuals/`: the page, volume 2 (depth 5–20, untested) and the jev
    manual;
  - `knowledge/`: each run's notebook;
  - the `.md` write-ups of `records/`.

  `refresh-sources.py` has a new `suffix` selection for the last one.
- **Recorded in:** SOURCES (S15, with what we take and what we leave),
  GOALS D11 (borg-agent and borg-runner, same terms), README, AGENTS,
  reference/README.
- **How to use it:** each FACT line is checked against `engine/angband/src/`
  and cited (RULES 16). An OBSERVED line rests on a few runs, some of which
  inherited each other's notebooks. Nothing is a claim until the zoo has
  measured it (RULES 14).
- **First use:** the strategies and profiles of M3 onward, alongside S14.

## 2026-11-10, end of day: M2's review continues tomorrow

**The review of M2 (#99) is tomorrow, 2026-11-11.** The owner runs it with
the reviewer they choose (RULES 6). #99's description has the prompt,
pinned to the head of `main`. Until it is in:
- nothing else merges into `main`, so that #99's diff stays what the
  review reads;
- #103 (borg-runner as source S15) waits, CI green. After #99 merges,
  merge `main` into it (no force-push) and merge it;
- M3 doesn't start.

When the review is in: if it is clean, the owner merges #99, `m2` is
tagged on the merge, the findings become M3 issues, then #103, then M3.
If it isn't, fix the findings in PRs into `main`, repin #99's prompt and
map the findings in a comment, as for the three passes so far (#100–#104).

## 2026-11-10: M2's third review: not done; findings fixed

Kimi K3.1 Pro's third review of #99, at `fb069d8`: **not done**. R2 and R4
are resolved. One blocking finding and three should-fix ones were new, all
fixed in the PR after #102:
1. **Blocking:** perception read `pack_size` inventory entries, but the
   pack has `pack_size + 1` and may be overfull by one. It now reads them
   all.
2. **Slots and levels:** the equipment's slot count and types are checked
   against the race's static body, and the occupied slots against the
   engine's; `Item.level` is compared.
3. **Pairing:** each engine object is paired with the observed item at its
   place and position. Each pair is compared field by field from the object
   directly, and whole against `analyze()` of that very object: exact
   weight, flags, modifiers, elements, description, awareness, runes. The
   description, awareness and runes come from the same engine functions as
   perception, so that half checks pairing and completeness, not those
   functions.
4. **The stat floor:** `wearing` takes the bonus nearest the known one
   (race, class, the gear's known modifiers). That is exact unless
   something unseen changes the stat at the floor. The reviewer's
   counterexamples are tests.

**Evidence:**
- Smoke and full on the test build: 76,175 thinks, 0 differences.
- The reviewer's faults now fail: slot types off by one, 48,948
  differences; zeroed item levels, 20,395.
- The normal build plays the references' games (3/3, 3/3, 45/45). Its
  deaths are D0943–D0988.
- **Not exercised:** no suite run has an overfull pack or mixed cursed and
  uncursed stacks.

## 2026-11-10: M2's re-review: done with changes; R1–R4 fixed

Kimi K3.1 Pro re-reviewed #99 at `eef7a8c`: **done with changes**. Both
blocking findings and the first round are resolved. Four remained, fixed
in the PR after #101:
- **R1:** the gear check is now place-aware. The engine says where each
  object is (`equipped_item_slot`, `object_is_in_quiver`), and each item
  is compared on its identity and known combat values, and known flags
  against bounds (superseded by the third review's pairing, below).
- **R2:** the pack count is compared with a fresh `pack_slots_used` call.
- **R3:** at the stat floor of 3, `wearing` takes the lowest result of all
  the bonuses that fit, so no stat is invented.
- **R4:** monsters are compared as multisets, so duplicates count.

**Evidence:**
- Smoke and full on the test build: 76,175 thinks, 0 differences.
- The reviewer's faults now fail: weapon and launcher swapped, 8,158
  differences; every monster duplicated, 48.
- The normal build plays the references' games (3/3, 3/3, 45/45). Its
  deaths are D0897–D0942.

**Next:** the owner merges #99 (or asks for another pass). Then tag `m2`,
the remaining findings (none open) become M3 issues, and M3 starts.

## 2026-11-10: M2's review (#99): not done; findings fixed

Kimi K3.1 Pro reviewed M2 at `70cb9b5`: **not done**, with two blocking and
six should-fix findings. They are fixed in #100 (findings 7, 8, doc nits)
and the PR after it (findings 1–6):
- **Finding 1 (blocking):**
  - **Fix:** the shop number and the web flag are read only where the
    player knows a shop or a floor, as the C Borg guards them.
  - **Check:** the truth check fails on any other shop or web.
- **Finding 2 (blocking):**
  - **Unknown grids:** a grid the player knows on the panel but the world
    doesn't is a difference.
  - **Panel:** the observed panel is checked against the engine's own
    `panel_contains`.
- **Finding 3:** monsters are compared field by field, and the gear item
  by item against the engine's gear list.
- **Finding 4:** every run needs one truth line per Borg decision, and a
  lost truth log fails the test build.
- **Finding 5:** `term_win` and `random` are in the types table, and a
  field-level audit fails on fields of unlisted types.
- **Finding 6:**
  - **To-dam:** the weapon and the launcher are left out of the gear's
    to-hit and to-dam; melee adds the weapon's own.
  - **Stats:** `wearing` recomputes the stats in use (`modify_stat_value`).
- **Findings 7–8 (#100):** `--runs` matches deaths by (family, keys);
  `make-reference.sh` refuses before writing anything.

**Evidence:**
- Smoke and full on the test build: 76,175 thinks, 0 differences.
- Kimi's three faults, which passed before, now fail: 219,103, 223,230 and
  96 differences.
- The normal build still plays the references' games (3/3, 3/3, 45/45),
  so the references stay. Its deaths are D0851–D0896.

**Next:** a re-review of #99 is the owner's call (RULES 6). #99's prompt is
updated to the new head.

## 2026-11-10: T2.5, perception against the engine; M2 complete

The owner closed E's review without a re-review. borg-zoo#48 now asks to
move `4.2.6-reborn-rust` to `f0eeee2`; the zoo check there gives the same
smoke play hashes as M1, and its death is D0804 (#97).

**T2.5 (#16):**
- **The check:** a test feature, `test-perception`, compares every think
  with the engine through other routes (`engine.rs`, below `ENGINE TRUTH`):
  - the world's remembered map against `player->cave`;
  - the engine's visible monsters on the panel against the monsters seen;
  - the gear list against the observed equipment, quiver and pack;
  - our pack count against `pack_slots_used`.
  `tools/check-perception.py` fails on any difference. It also checks the
  status against the frontend's `state.log`.
- **Results:** smoke and full, 76,175 thinks, have 0 differences.
  - Declared, not failed: 220,350 grid-thinks the player knows but the
    world never saw (218,978 in full, 1,372 in smoke), and 96 visible
    monsters off the panel. The C Borg
    reads only the panel too (`borg-update.c:264`).
  - An injected fault gives 8,205 differences, so the check can fail.
- **One defect found:** while hallucinating, `map_info` puts `m_idx = 1`
  on an empty grid. When monster 1 stood there unseen, the Borg took it as
  seen; 11 of 45 full runs failed the hallucination assertion, and CI had
  run only smoke. Perception now also needs `monster_is_visible`
  (`map_info`'s own test, a B read). Our suites play the same games
  (references unchanged: 3/3, 3/3, 45/45), since none of their runs met
  that case.
- **D3:** reason T for the check's reads, used only below the marker
  (`test_reads.py`). The types read field by field are listed (E review,
  finding 10).
- **CI:** one test build with both features runs smoke and full; its play
  equals the references, and `check-perception.py` passes.
- **Deaths:** D0805–D0850.

**M2 is complete:** T2.1–T2.5. Next is the review PR (`main` → `reviewed`)
with its prompt; the owner chooses the reviewer (RULES 6). M3 doesn't start
before that.

## 2026-11-10: E's review is in; fixes in three PRs

Kimi's review of E (on #92) has ten findings. They are fixed in three PRs
into `main`:
1. **The frontend (#95, RULES 22):** `loaded` and `exact_turns` are in
   every summary (finding 5).
   - Play doesn't change: keys, messages and states equal the references
     (smoke 3/3, smoke-exact 3/3, full 45/45).
   - `harness/diff` compares the two fields when both sides have them, as
     it does `game_options`.
   - Its deaths are recorded at its source (D0712–D0757).
2. **The code (#96):** suites, perception while hallucinating, the death and
   reference tools.
   - References are regenerated: the full suite's `menu` fix changes
     exactly the 20 runs whose character changes.
   - Smoke, smoke-exact and the other 25 full runs are unchanged.
   - Its deaths are D0758–D0803.
3. **Documents, rules and CI (#94):**
   - findings 6, 7, 9 and 10's document nits;
   - RULES 12 narrowed by the owner: measured runs only, not watched games;
   - CI fetches upstream `e473720`, so the tree check no longer skips.

**Left:**
- ENGINE-READS' types table (`term`, `monster`, `player_class`, `store`):
  T2.5's D3 recheck (#16).
- borg-zoo#48: `4.2.6-reborn-rust`'s `ref` moves to the commit that merges
  #94, not to `e`, since #96 changes the full suite's play (the zoo's own
  cells pass `-race`/`-class` with `-menu-letters`; whether its play
  changed is for the zoo's check).
- Whether the fixes get a re-review is the owner's call (RULES 6).
- Then T2.5 (#16) and M3.

## 2026-11-09: E merged before its review; a review after the fact

The owner had the session merge #92 (`main` → `reviewed`). Tag `e`
(`0a92da0`) was pushed, milestone E closed and borg-zoo#48 filed (move
`4.2.6-reborn-rust` to `e`), all **before any review**. The owner then
asked for the review after the fact:
- the reviewer, the owner's choice, reviews `55d7476` (the tree of `e`) and
  posts on #92;
- findings are fixed in PRs into `main`;
- tags are not moved (RULES 5);
- borg-zoo#48's first request is on hold until the review is clean.

Other work (T2.5, #16) may go on meanwhile.

## 2026-11-09: TE.9 and milestone E complete; E's review

- **TE.9:** `engine/tests/test_independence.py` (CI, rust job) checks that
  no build, test, check or CI step uses borg-rs. That means no checkout,
  crate, staticlib or former `angband/` tree, and no clone or download;
  provenance comments are allowed. It also checks that the workspace holds
  only our crate. CI now proves every point of SPEC §10's exit:
  - the tree is upstream's, the patches apply, and the source list matches;
  - the frontend contract, the zoo contract and the D3 list hold;
  - the bindings regenerate unchanged;
  - runs are deterministic, match the references, and keep the boundary
    contract;
  - the death reconciliation passes, and a CI death reruns with the same
    keys;
  - nothing reads borg-rs.
- **Milestone E is complete** (#81–#91). The review PR `main` →
  `reviewed` is open. The owner chooses the reviewer (RULES 6); the prompt
  is in the review PR's description.
- **After the owner merges it:**
  - tag `e`;
  - ask borg-zoo (an issue, RULES 7) to move `4.2.6-reborn-rust`'s `ref`
    to `e^{}` and to decide on the two C specimens;
  - continue with the rest of M2 (T2.5, #16), then M3.

## 2026-11-09: S12 retired (pin PR, #78)

The owner retired S12, the draft strategy manual, in favour of S14, the
complete one. This pin PR (RULES 21) removes the `strategy-manual` pin, its
copy rule and `reference/strategy/`, and updates SOURCES, reference/README
and AGENTS.md. Its local clone, `vendor/neo-angband-tracker-workbench-c-borg`,
is no longer used.

## 2026-11-09: TE.8, removals, documents and rules (#78)

- **Removed:**
  - the C arms' references (`harness/reference/smoke-c`, `smoke-pristine`,
    `full-pristine`, `smoke-c-bbc-m7`);
  - `pins.json`'s `controls` and borg-rs's oracle role, `borg_dir` and
    `tag_of_interest`;
  - `docs/ENGINE-TREE.md` (replaced by `docs/ENGINE.md`);
  - `suite.sh`'s dead control-arm branch.
- **Rewritten for docs/SPEC.md:**
  - RULES 9 (the tree is upstream's bytes plus our patch series), RULES 16
    (copies are recorded in PROVENANCE and owned) and RULES 22 (comparisons
    are the zoo's; a change to the integration reproduces the references
    or explains every difference);
  - GOALS principle 4, D2, D4 and D8, plus a superseded note on the "same
    seam, same harness as borg-rs" principle;
  - ARCHITECTURE L0, *What comes from where*, and the settled open
    question;
  - SOURCES S5 (borg-rs is a reference), with a historical note on its
    section;
  - README (intro, status, SPEC first in the table);
  - AGENTS.md (intro, start commands, the `engine/` and `harness/` rows,
    the `borg-rs/` row gone);
  - `harness/README.md` (rewritten for our harness);
  - the refresh report's wording (a new better-borg-c tag is a specimen for
    the zoo to consider).
- History stays history: PLAN's M1 row, the handover's earlier entries,
  `docs/ZOO-CHANGE-REQUEST.md`, and the M1-era rows of SOURCES.
- **S12's retirement** is its own pin PR (RULES 21), next.

## 2026-11-09: TE.7, the zoo contract (#77)

- **`docs/ZOO-INTERFACE.md`:** the contract the zoo's borg-rs adapter
  (`zoo/borgrs.py` at our pin `e56d5fa`) calls. It covers the build
  (`harness/build.sh`, `build/rust/game/`, `BUILD_ID`'s `binary_sha256` and
  `rustc`, `harness/borg.txt`), the run (`-mborgh` and its flags,
  `TZ`/locale), and what the zoo reads (`summary.json`'s 10 fields,
  `keys.log`, `state.log`). One mismatch is on the zoo's side: its `STOP`
  map knows `clev`, while our frontend writes `clevel`. It's harmless,
  since the zoo never passes `-clev`.
- **`engine/tests/test_zoo_contract.py`:** runs the binary with exactly the
  zoo's command line and checks every field and log format, plus that
  `BUILD_ID` matches the binary and another `BORG_IMPL` fails loudly. CI's
  engine job runs it.
- **`harness/zoo-check.py`:** checks only the Rust arm. End to end with the
  zoo's adapter: 3/3 cells with no harness failure or config error, and
  **play hashes identical to M1's**. The seed-29 death is recorded as D0711.
- **Retired:** `tools/endpoint.py` and its test (borg-zoo computes every
  statistic). GOALS D7 is amended.
- **After E is merged:** ask borg-zoo (an issue, RULES 7) to move
  `4.2.6-reborn-rust`'s `ref` to E's tag and to decide on the two C
  specimens.

## 2026-11-09: TE.6, our death tools; the copied tooling and borg-rs's tree gone (#76)

- **Decision (owner):** the death tooling to keep imported borg-rs's whole
  recording toolkit (GUI, replay viewer, runtime: about 2,000 lines), so
  all copied tools are dropped and replaced by lean tools of ours.
- **New:**
  - `tools/rerun-death.py` reruns a dossier's game from its build and seed
    with the observation, world and knowledge logs and checkpoint saves,
    and checks that the keys are the recorded ones. D0664 reproduces, and
    CI reruns one of its own deaths.
  - `tools/death_registry.py` regenerates the registry index, taken from
    `borg_deaths.py` with the same output (byte for byte).
  - `watch-borg.sh` builds our integration with X11 and opens the game.
- **Removed:**
  - the copied tools and their tests (25 Python files, 2 C files, 2
    headers) and 7 root launchers;
  - borg-rs's `angband/` tree, the `borg-rs/` shim crate and
    `harness/measurement/`;
  - `pins.json`'s `engine_tree` and the re-sync report;
  - the repository test `engine_tree_matches_its_record`;
  - `run.sh`'s `frames` flag.

  No file of borg-rs's engine tree is left, and no build reads one.
- **Docs:** README's *Playing*, AGENTS.md, `tools/README.md`,
  `docs/PROVENANCE.md` and SPEC §8 and §11 are updated.
- **Deaths:** the Rust deaths are recorded again, as D0665–D0710
  (`Cargo.toml` is a build input and lost the shim member). Play still
  matches the references: full 45/45, smoke 3/3.

## 2026-11-09: TE.5, the harness and run contract on our build (#75)

- **`run.sh`:** the C Borgs' flags (`cheat`, `cheatlive`, `munchkin`,
  `lunal`, `selflunal`, borg.txt rewrites) and the dropped frontend options
  (`frames`, `rune-pattern`) are gone. `BORG_IMPL` defaults to `rust`, and
  the run's `user/borg.txt` is written as before (the zoo copies the file;
  its hash is in a death's family). The environment is fixed (`TZ=UTC`,
  locale).
- **Option set:** the frontend writes every game option in force into
  `summary.json` (`game_options`), and `docs/ENGINE.md` declares the Rust
  Borg's options against the C Borg's.
  - **Found:** 4.2.6 has no `easy_alter`. The Rust Borg had been setting it,
    and the engine ignored it silently. It's removed, and an unknown option
    now stops the Borg (`GAME_OPTIONS` in `borg.rs`). Play can't change,
    since the call never did anything.
- **Reproduction:** the Rust Borg on our build reproduces the borg-rs-tree
  outputs: full 45/45, smoke 3/3, smoke-exact 3/3, hallucination smoke 3/3.
  `harness/diff` compares `game_options` only when both sides have it.
- **References:** `harness/reference/smoke-rust`, `full-rust` and
  `smoke-exact-rust`, made by `harness/make-reference.sh` with a
  `REFERENCE` file (source, binary, profile, borg.txt hashes,
  environment). CI compares smoke and smoke-exact with them. The C arms'
  references go in TE.8.
- **Deaths:** the Rust deaths are recorded again at the new source, as
  D0619–D0664.

## 2026-11-09: TE.4, our build (#74)

- **The build:** `engine/CMakeLists.txt` builds the Rust Borg from the
  pristine tree, our patches, our frontend and the crate's own staticlib.
  `harness/build.sh` (`BORG_IMPL=rust`) uses it and is rewritten; the build
  takes about 10 s.
- **Reproduction:** the Rust Borg plays **byte-identical** keys, messages
  and states to its build on borg-rs's tree (smoke 3/3, full 45/45,
  smoke-exact 3/3, hallucination smoke 3/3). The boundary contract holds.
  So the patch series and our frontend are a complete harness.
- **Gone:** the C arms (`c`, `c-pristine`, `c-bbc-m7`) from the build and
  CI, and `harness/control-overlay.py`. `harness/identity.sh` knows only
  `rust`, and its inputs are `engine/`, the crate, `Cargo.toml` and
  `Cargo.lock`.
- **Kept until TE.6:** borg-rs's `angband/`, the `borg-rs/` shim crate and
  `harness/measurement/`. The copied death tooling still builds its own
  binary from them (#76 updated).
- **Tests:** `engine/tests/test_build.py` checks the source list against
  upstream's and the input list, and runs the binary through `run.sh` for
  the frontend contract. CI's engine job runs it.
- **Deaths:** the Rust deaths are recorded again at the new source, as
  D0573–D0618 (the build inputs changed).

## 2026-11-09: TE.3, generated bindings and the D3 list (#73)

- **Generated bindings:** `borg-reborn-rs/src/bindings.rs` is generated by
  bindgen 0.73.2 from the pristine headers (`engine/bindings/`), limited to
  the symbols the Borg uses. It has 66 compile-time layout assertions
  against the C structs. The c2rust `headers/` copied from borg-rs are
  gone.
- **`engine.rs`:** re-based on the bindings. Its hand-written declarations
  of engine symbols are gone, except `inkey_hack`, which no upstream header
  declares.
- **The D3 list:** `docs/ENGINE-READS.md` lists every symbol with its
  reason (screen, direct read, static data, declared extra, write, seam).
  `engine/tests/test_reads.py` checks it both ways, and that only
  `engine.rs` touches the bindings.
- **CI:** installs libclang and bindgen and runs `generate.sh --check`.
  Locally this needed `clang`, `libclang-dev` (owner, apt) and `cargo
  install bindgen-cli`.
- **Play:** unchanged (Rust smoke 3/3, full 45/45). The Rust deaths are
  recorded again at the new source, as D0527–D0572.

## 2026-11-09: TE.2, our headless frontend (#72)

- **The frontend:** `engine/frontend/main-borgh.c` and `.h` are copied
  from borg-rs's borgh and trimmed to the contract of SPEC §3.5 (1,247 →
  931 lines). Dropped:
  - `-states` (native observer);
  - `-frames` and its diagnostics;
  - fixture building (`-make-fixture`, `-near-morgoth`);
  - `-stop-rune-pattern`;
  - `-cheat-live` (D5).

  Kept: every contract flag, the stall and key limits, `-clev`,
  `-random-birth`, the save options and checkpoints (the death tooling may
  use them).
- **The test:** `engine/tests/test_frontend.py` checks the flags, the
  `summary.json` fields, the mode registration, and a compile against the
  patched tree.
- **Not yet:** no binary until TE.4. `harness/run.sh` flags that used the
  dropped options (`frames`, `cheatlive`, `rune-pattern`) are cleaned up
  in TE.5.

## 2026-11-09: TE.1, the upstream tree and our patch series (#71)

- **The tree:** `engine/angband/` is upstream `e473720`, byte for byte
  (tree `99a0549`, `engine/UPSTREAM`), extracted from upstream, not from
  borg-rs.
- **The patches:** `engine/patches/` holds 7 patches for our harness hooks,
  every changed line guarded by `BORG_HARNESS` or `USE_BORGH`, plus one
  declared split of a declaration in `mon-make.c`.
- **Docs and tests:** `docs/ENGINE.md` and `docs/PROVENANCE.md` are new.
  `engine/tests/test_engine.py` checks the tree, that the series applies,
  and that with the guards off the code is upstream's. It runs in CI.
- **What stays:** borg-rs's `angband/` stays until TE.4 switches the build;
  nothing builds from `engine/` yet.

## 2026-11-09: milestone E planned

PLAN.md follows docs/SPEC.md §10. **E. Our own integration** is in
progress: the GitHub milestone E, tasks #71 (TE.1) to #79 (TE.9), linked by
their Needs. M2 is paused (T2.1–T2.4 done; T2.5, #16, needs E). Order:
- TE.1 tree and patches, then TE.2 frontend and TE.3 bindings;
- TE.4 build, then TE.5 harness and the reproduction check;
- TE.6 death tooling, TE.7 zoo contract and TE.8 removals and documents;
- TE.9 CI and E's review (the owner chooses the reviewer).

## 2026-11-09: the spec is in effect; only our Borg

The owner merged #66 (`main` → `reviewed`, `37bcb97`), so `docs/SPEC.md`
is in effect. There is no tag: it is not the end of a milestone. The owner
then decided:
- **borg-zoo does every comparison.** This repository delivers only our
  Borg, through the zoo contract. The C arms (`c`, `c-pristine`,
  `c-bbc-m7`), their overlays and references, `tools/endpoint.py` and #56
  (`c-bbc-m8`, now borg-zoo#36) are dropped in milestone E. GOALS D7 and D8
  change accordingly.
- **§11:** keep the death tooling and drop the GUI, replay and takeover
  tools; keep the zoo's current interface, written down; the engine-change
  question goes with the controls.

The `borg_self_scum = TRUE` finding concerns the zoo's
`4.2.6-reborn-c-*` specimens, which were built from this repository. It
is reported on borg-zoo#30.

Next: plan milestone E as issues (PLAN.md re-planned), then build it.

## 2026-11-09: the owner chooses the reviewer (RULES 6)

The review of #66 was started by the session: MiniMax-M3, picked through
the quota service. The owner then ruled: "next time give me the prompt and
I will chose the reviewer". RULES 6 and AGENTS.md (*Models*) now say so.
For every review the session writes the prompt and the owner chooses and
runs the reviewer. A second pass on #66 (for #67 and #68) therefore waits for
the owner's choice.

## 2026-11-09: review of #66 (MiniMax-M3): spec findings

The spec findings S1–S10 and the missed risks are answered in
`docs/SPEC.md`:
- **S1:** the zoo contract is written down. It covers the build entry point
  and layout, `harness/borg.txt`, and the frontend's mode `borgh`, flags
  and `summary.json` fields. The harness may change everywhere else.
- **S2:** the frontend registration (`main.c`, `main.h`) joins the patch
  series, and the line counts are corrected.
- **S3:** our CMake carries a copy of upstream's source list, checked
  against it; upstream's file can't be used as it stands.
- **S4, C2:** the configuration (the C Borgs' `borg.txt` with every D5
  exploit off, the game option set) and the environment (`TZ`, locale)
  become part of the run contract.
- **S5:** E's exit requires the new integration to reproduce today's
  `c-pristine` and Rust smoke and full keys before any old reference is
  replaced.
- **S6:** `docs/ENGINE-READS.md` is normative; channel A gains the
  message line's attribute and cursor; the four extra reads are declared.
- **S7, S9:** the inventory and the document lists are completed.
- **S8:** `c-pristine` is a control, not an oracle.
- **S10 and the risks:** configuration hashes and what CI must prove are
  in E's exit; the death registry's two generations are described.
- The reviewer's answers to §11 are recorded as recommendations; the owner
  decides.

**A finding for the owner (S4):** `harness/borg.txt` has
`borg_self_scum = TRUE`, upstream's default too. So in every control run so
far, a C Borg could money-scum for a ware from character level 10
(`borg-store-buy.c:141-153`), which D5 excludes. Its effect on M1's
numbers (c-pristine 1/45 and c-bbc-m7 2/45 at the full-exact cap) has not
been measured. The configuration is corrected in milestone E.
## 2026-11-09: review of #66 (MiniMax-M3): code findings

MiniMax-M3 reviewed the spec and the work since `m1` on #66: the spec is
"yes, with changes", and the code is sound. Code findings:
- **C1, fixed here:** `Character::blows` now follows the engine for a
  weapon too heavy to wield (one blow, `calc_bonuses:2296-2318`, with
  `adj_str_hold` lifted and checked against the engine's copy), and adds
  bloodlust's `timed / 20`. No recorded run heavy-wields yet, so the rule
  is unit-tested only.
- **C5, fixed here:** `with_borg_rng` restores the game's RNG through a
  drop guard, on every path.
- **C2** (the Rust Borg's game options differ from the C Borg's) goes to
  the spec's run contract.
- **C3** (a key aborts the Borg in a shop, which the C Borg does not) is for
  M3, when the Rust Borg first enters shops.
- **C4** (flushing the trace each think) is kept: a dossier needs the trace
  on disk when the game ends, and one write costs microseconds against
  borgh's stall limit of seconds.

The Rust deaths are recorded again as D0480–D0525.

## 2026-11-09: the spec (docs/SPEC.md), and a stop

The owner decided to move completely away from borg-rs. The game stays
upstream C Angband. Our own integration (hooks, frontend, bindings, build,
harness) is built first, and borg-rs is only a reference to copy from.
Before anything more is implemented, `docs/SPEC.md` states that project:
- the parts it keeps, replaces or drops (§8);
- the documents and rules that change (§9);
- a re-plan with a new milestone E, our own integration, first (§10);
- four open questions (§11).

The owner accepted it and sent it to review with the work since `m1`. It
takes effect when the owner merges that review PR. Until then: **no
implementation**, and the M2 task list after T2.4 (T2.5, then M3+) is on
hold for the re-plan.

## 2026-11-09: owner decisions

Asked again and answered on 2026-11-09:
1. **c-bbc-m8** (#56) runs with `borg_munchkin_by_class = FALSE`, so every
   arm keeps D5. Its label must say so. GOALS D8 is amended.
2. **The `smoke-c-bbc-m7` reference** is accepted as a self-produced
   regression reference (harness/README.md).
3. **S12 is retired** in favour of S14. The pin, its copy rule and
   `reference/strategy/` go in their own pin PR (RULES 21).
4. **T2.5: the engine is the truth, not borg-rs** (option D). borg-rs's
   perception can't be run on its own: it lives inside its decision loop.
   Perception is checked against engine facts reached by other routes
   than the Observation's reads, and every engine read is listed (the D3
   audit). GOALS D8, ARCHITECTURE L1, AGENTS.md and #16 are amended.

## 2026-11-09: T2.4, decision trace and parameter table (#15)

- `params.rs`: the parameter table has name, unit, range, default and
  the evidence for the default. It holds the minimal Borg's three numbers:
  `max_tries_per_turn` 2, `rest_below_hp_pct` 70 (integer percent, so
  `hp * 100 < max * 70` is the old test exactly) and `rest_turns` 75
  (`Action::Rest(n)`, whose keys are still `R75` Enter).
- Profiles are `name = value` files in borg.txt's syntax, read at start
  (`BORG_PROFILE`; run.sh defaults to
  `harness/profiles/reborn-default.txt`). An unknown name, a malformed
  line or an out-of-range value stops the game.
- The profile's SHA-256 is in `config.txt` (run.sh), `params.json` and
  every trace line.
- `trace.rs` plus `decide_with`: the decision now carries its goal,
  budget, scored candidates and violations; the guard of principle 6 is
  recorded as one. The Borg writes `params.json` and `trace.jsonl` to
  the run directory (`BORG_RUN_DIR`, from run.sh, Rust arm only). The trace
  is written after the decision from what the decision made, so play is
  unchanged (full 45/45, smoke 3/3, hallucination smoke 3/3 identical).
- Dossiers keep `params.json` and `trace-tail.jsonl` (the last 200
  thinks), and `record.json` gains `identity.profile_sha256` and
  `decision_trace`. CI checks the trace and parameters in run outputs and
  in Rust dossiers.
- The Rust deaths are recorded again as D0434–D0479, now with traces. The
  C arms' outputs and dossiers are unchanged.

## 2026-11-09: T2.3, knowledge tables (#14)

`borg-reborn-rs/src/knowledge.rs` holds the L3 `Knowledge`. The Borg reads
it once from the engine's parsed data at its first think
(`engine::knowledge()`):
- monster races (spoiler data, GOALS D3: level, speed, HP, AC, sleep,
  frequencies, race and spell flags, and blows with method, effect, power
  and dice);
- object kinds, egos and artifacts;
- classes with their blows parameters and spell books;
- player races;
- what each store always and normally stocks;
- the engine's `extract_energy` table and `move_energy`.

Lifted, with citations at the pinned engine:
- `stat_index` (`calc_bonuses:2065-2073`);
- `blows` (`calc_blows:1703-1735`, with `adj_str_blow`, `adj_dex_blow` and
  `blows_table`). better-borg-c milestone 8's fix to the C Borg's copy
  (`668ac32`, weigh one weapon, not the stack) agrees with the engine, so
  the engine's formula is used.

Uses:
- `Knowledge::turns_per_action` replaces the placeholder speed rule in
  `Track::confidence`;
- `Character::blows` and `melee_per_turn` give expected damage per turn for
  any weapon (blows times mean dice plus known to-dam, before hit chance,
  criticals and slays, which belong to M4's threat model).

The Observation gained the engine's own `stat_ind`, `num_blows` and the
O-combat option, as checks. Tests:
- unit tests use the C values worked by hand;
- `BORG_KNOWLEDGE_LOG=FILE` dumps the knowledge, and CI checks it against
  monster.txt and class.txt values and against the engine's `adj_str_blow`;
- on every recorded think our stat indices and blows equal the engine's:
  locally 595 thinks over all 9 classes, and in CI the Warrior run.

Play is unchanged (smoke 3/3, full 45/45, hallucination smoke 3/3). The
Rust deaths are recorded again as D0388–D0433.

## 2026-11-09: T2.2, the world model (#13)

`borg-reborn-rs/src/world.rs` holds the L2 `World`. It changes only in
`World::update(&Observation)`, once per think. It has:
- the current level's map memory: the feature last shown, when the grid
  was last in view, the remembered and the real top object, trap and web.
  An unknown grid never erases what was known. Leaving the level (a depth
  or level-size change) forgets it, as the C Borg does;
- monster tracks keyed by the game's monster index: race, position, first
  and last seen, HP, timers and speed. An index reused for another race is
  a new monster. A track out of view is kept, and `Track::confidence`
  falls with the time since it was seen and with the monster's speed
  (asleep, a quarter as fast). Its shape is a placeholder for M4's
  calibration;
- the town's shop entrances (kept across levels) with the last visit (the
  store header "Stor"/"Home" while standing on the entrance), and every
  store's stock and the home's, as last read;
- the character model: the latest status and gear, plus `GearBonuses`
  (to-hit, to-dam, AC, known flags and modifiers, and resistances combined
  as `calc_bonuses` does). `Character::wearing` gives a changed copy for
  "try this on", and `pack_free` uses the engine's own count.

The Observation gained the engine's pack count (`pack_slots_used`,
`z_info->pack_size`) and every store's stock (`borg_notice_store`: each
ware through the item analysis, with the shop's `price_item`).
`BORG_WORLD_LOG=FILE` writes the world's digest after every think. CI
records a 3,000-turn run with both logs, and checks that replaying the
observations without the game rebuilds the live world think by think (302
of 302 locally). Play is unchanged (Rust smoke 3/3, full 45/45 and
hallucination smoke identical), but the store reads and the update make the
suites about 25% slower. The Rust deaths are recorded again as
D0342–D0387.

Left for later tasks: expected damage per turn needs blows and the other
formulas of L3 (T2.3). Item curses and activations are not observed yet.
The strategy doesn't read the world until M3.

## 2026-11-09: T2.1, the Observation (#12)

`borg-reborn-rs/src/observation.rs` holds the L1 `Observation`. It is
plain, serde-serialisable data, one per think, with what the C Borg's two
channels give it (spec §1.1, GOALS D3):
- from the screen: the message line (row 0, split into messages by
  `parse_messages`) and the store header (row 3, columns 1–4, raw: the
  C Borg compares it with "Stor"/"Home");
- from direct reads: status and timed effects, every equipment slot, the
  quiver and the pack through the C Borg's item analysis (`ODESC_FULL`
  descriptions, the known part unless the item is fully known), the
  class's spells (`spell_flags`, base fail), the map panel through
  `map_info()` (clipped to the level, with known trap, shop number and
  web), the monsters `map_info()` shows with their real race, hit points,
  timers and speed, and the real kind of the top object that is not
  ignored on each grid with a seen object.

`engine::perceive()` does the reads, so `engine.rs` is still the only
`unsafe` module. The Borg perceives on every think, after the minimal
Borg's own reads. The placeholder strategy does not use the observation
yet, and play is unchanged (Rust smoke 3/3, full 45/45 and hallucination
smoke identical). `BORG_OBSERVATION_LOG=FILE` writes one JSON line per
think, about 270 KB each. CI records a 500-turn run and checks that every
line round-trips canonically. New dependencies: `serde` and `serde_json`.
The Rust deaths are recorded again as D0296–D0341.

Not in it yet, for T2.2–T2.5: the store's own inventory (when the Borg is
in a shop), item curses and activations, monster messages beyond the
line, and the oracle comparison with borg-rs's `BorgState` (T2.5).

## 2026-11-09: engine tree re-synced from borg-rs 286215c

`angband/` is now borg-rs `286215c:angband` (tree `3fc0c9f`, split
`6581b4f`), merged as a subtree (docs/ENGINE-TREE.md). Only
`angband/src/borg/` changed: borg-rs carried better-borg-c milestones 7
and 8. So the tree's own C Borg (`BORG_IMPL=c`, never a control) goes
from impact v6 to v8. It includes `borg_munchkin_by_class`, TRUE by
default: Warriors, Druids and Rangers take the munchkin start. The engine
and borg-rs's generated headers are unchanged.
- `harness/reference/smoke-c/` is replaced by a smoke run of borg-rs's own
  C build at `286215c`. Ours reproduces it byte for byte (3/3), the same
  import check as T1.2.
- A re-sync starts a new measurement baseline (RULES 22). The source hash
  of every arm includes `angband/`, so deaths from runs after this PR
  are new encounter families.
- `harness/borg.txt` is unchanged. Whether a control should run with
  `borg_munchkin_by_class = FALSE` (GOALS D5) is asked on #56: better-borg-c
  v8 has the option too.

## 2026-11-09: M2 source refresh

The M2 `--update` (RULES 21) moved five pins:
- borg-rs `e5cb263` → `286215c`: two docs changed, and the engine tree
  re-sync is available (next PR);
- better-borg-c `b1588cc` → `eb9a78e`: new deaths D0015 and D0016;
- strategy-manual `e67fd7d` → `86349ca`;
- borg-zoo `3f6b76e` → `e56d5fa`;
- borg-agent `9c58125` → `77e48c9`.

angband and neo-angband-tracker are at their pins.
- **S12 repointed**: the pin `strategy-manual` now follows
  neo-angband-tracker `workbench/c-borg` (its own clone
  `vendor/neo-angband-tracker-workbench-c-borg`), since `c-borg-reference`
  is now pristine upstream. The manual is complete there: chapter 3
  (`03-magic.md`) is new. S12 stays the draft of S14 until the owner
  retires it.
- **New deaths** (better-borg-c, both CL1 Necromancers, original-C
  mechanisms):
  - D0015: Fang, in a dark dead end seven squares from an up staircase.
    The C Borg forbids the stair dash from Grip or Fang, while another rule
    lists Fang as a reason to leave the level.
  - D0016: a fast crow after a teleport trap. The starting gold went on
    armour and unidentified rings, Recall stayed at home, and two known
    Cure Light Wounds were never offered while fleeing.

  Both are input for M4 (escapes, the tactical layer: retreat to stairs,
  healing while fleeing) and M5 (readiness: a survival stock before
  descending).
- **borg-zoo** at `e56d5fa` has borg-zoo#30 done (borg-zoo#32): the
  specimens `4.2.6-reborn-rust`, `-c-pristine` and `-c-bbc-m7` use
  `ref` `2dd11fd`, with `exact_turns`, the depth stream and kept death
  evidence. A comment on borg-zoo#30 gives them the `m1` commit
  (`fab7155`), and says what changed since `2dd11fd`: no keys change in
  our suites, but the Rust source hash does.

## 2026-11-09: a new control arm, c-bbc-m8 (#56)

better-borg-c tagged milestone 8 (`4.2.6-impact-v8`, `017e038`) before
our better-borg-c pin moved past it. The refresh report listed only tags
after the pin, so it never showed the tag. It now lists tags after the
baseline in use (`baseline.commit`, else the pin), which flags v8. Under
RULES 22, v8 is a new arm, `c-bbc-m8`, filed as #56 in M4, the first
milestone that measures against the controls. `c-bbc-m7` stays as it is,
and the GOALS wording "`c-bbc-m7` today" changes when #56 lands.

## 2026-11-09: M1 done; M2 starts

The owner merged the M1 review PR #39 (`main` → `reviewed`, merge
`fab7155`) after four review passes by Kimi K3.1 Pro. Tag `m1` is
annotated and points to that commit. Its message says "merged 2026-11-10",
following the dates of the entries below, but the merge was on
2026-11-09; tags are not moved (RULES 5). Those entries, marked
2026-11-10, are from 2026-11-09 too. The GitHub milestone M1 is closed,
and PLAN.md marks M2 in progress.

After the review, as the owner decided:
- RULES 7 now lets a session open issues, and keep its own issues up to
  date, on borg-zoo, borg-rs, better-borg-c, neo-angband-tracker and
  borg-agent, when the owner asks or when this repository needs a change
  there. Nothing is pushed or merged there, and angband/angband stays
  off limits.
- The zoo change request is
  [diegoami/borg-zoo#30](https://github.com/diegoami/borg-zoo/issues/30).
  docs/ZOO-CHANGE-REQUEST.md names the `m1` commit `fab71554fd32`.
  The issue's prompt is updated to that exact commit after this PR is
  merged, because its old fallback (`4236081`, unpeeled tag) is obsolete.

Found while re-reading RULES: commits and PR bodies since M0 carried
`Co-Authored-By` and "Generated with" lines, which RULES 4 forbids. They
stay, since published commits are not amended (RULES 5), and no new
commit has them.

Next, in M2 (each its own PR):
1. The source refresh: `--update` for better-borg-c (new deaths D0015 and
   D0016) and the other moved pins. Repoint `strategy-manual` (S12) to
   neo-angband-tracker `workbench/c-borg` so the copy keeps a source; the
   owner may still retire S12 in favour of S14.
2. The borg-rs impact-v7 engine re-sync (docs/ENGINE-TREE.md).
3. Then #12 (T2.1, Observation) and the rest of M2 in order.

Still open for the owner: whether the self-produced `smoke-c-bbc-m7`
reference is acceptable, and S12's retirement.

## 2026-11-10: M1 fourth review pass, findings S1–S2 (#39)

The fourth pass (Kimi K3.1 Pro, at `eff27c9`) resolved R1, R2, R4, R5 and
R6, and partly resolved R3. It added two should-fix findings in R3's new
extra-option load path; nothing blocks. Both are fixed in #54. run.sh now
handles `-- -load FILE` like `save=NAME`: it copies the file to
`OUT/save/input.save`, passes that absolute path to borgh, and drops `-n`.
Before, borgh received `-n`, so it started a new character that the summary
called loaded (S1). A relative path also failed after run.sh's `cd` (S2).
`harness/check-extra-load.sh`, now in CI, checks that a relative extra
load resumes the saved game and plays the same keys as `save=NAME`.

## 2026-11-10: M1 third review pass, findings R1–R6 (#39)

The third pass (Kimi K3.1 Pro, at `42ee347`) resolved N1, N3, N4, N5 and
N8, and resolved N6 and N10 for their cases. It partly resolved N2, N7, N9
and N11, and added six should-fix findings, R1–R6. Nothing blocks. The fixes
go in three PRs into `main`:
- #51, R2: a step off a shop can be refused without time (an afraid player
  walking into a visible monster), so on a shop `stand_still()` now alters
  a neighbour (`+` and a direction), which always spends a move's energy.
  Rust play is unchanged (full 45/45, smoke 3/3, smoke-exact 3/3
  identical); the deaths are recorded again as D0188–D0233.
- #52, evidence: R1, R3 and R4. check-boundary compares `depths.json`'s
  `stop_turn` with the summary. The death exporter takes a loaded save
  from borgh's `loaded`, so `run.sh ... -- -load FILE` counts, and run.sh
  records that save's hash and an extra `-keep-load-rng` in `config.txt`.
  zoo-check imports the zoo from a `git archive` of the pin, not the
  checkout, so an ignored `.pyc` cannot replace pinned code.
- sources (this PR): R5 and R6. The update snapshot backs up into a fresh
  directory of its own and never touches an existing `.refresh-backup-*`.
  The zoo prompt peels annotated tag `m1` to its commit (`m1^{}`). The
  copy on borg-zoo#30 needs the same fix, with N11's, when its link is
  recorded.

## 2026-11-10: M1 re-review findings N1–N11 (#39)

The re-review (Kimi K3.1 Pro, at `2dd11fd` and `16af1ad`) resolved 12 of
the 19 findings, partly resolved 7, and added N1–N11. N1 blocks. The fixes
go in PRs into `main`:
- measurement: N1, N3, N5 and N6. `suite.sh` writes each run's
  `run_exit`. `tools/endpoint.py` scores no run of a suite whose binary
  changed (`binary_changed`), and in a `FAILED` suite only a run whose
  run.sh succeeded. A `depths.json` that contradicts `summary.json` is an
  `evidence_conflict`. It refuses outputs with no DONE or FAILED marker,
  and never guesses the class of a run when its index played two classes.
  `harness/check-boundary.py` checks the suite hash and fails survivors
  that stopped before the cap.
- #48, N2: on a shop entrance `,` enters the store and takes no time, so
  `stand_still()` steps off instead of holding. Rust play is unchanged
  (full 45/45, smoke 3/3 identical), but the source moved, so the Rust deaths
  are recorded again as D0142–D0187.
- #49, death exporter: N4, N7 and N8. `Interrupting` and empty causes are not
  natural; `keep-load-rng` drops the seed only with `save=`; a random
  birth's terminal record names the played indices.
- sources (this PR): N9, N10 and N11. zoo-check refuses a modified zoo at
  its pin; `--update` rolls back a failed phase two; the zoo prompt needs
  tag `m1` or an exact commit from the owner. The prompt already on
  borg-zoo#30 still has the old fallback, so update it when the link is
  recorded.

**Source moved (owner, 2026-11-10):** neo-angband-tracker's
`c-borg-reference` branch is now pristine upstream (`e473720`), with no
`docs/strategy/`. The manuals live on `workbench/c-borg` (`842dc5d`):
`docs/user-manual.md` and `docs/strategy/`. The S12 pin `e67fd7d` is still
reachable from `workbench/c-borg` (and from `docs/strategy-manual`, which
the owner may delete). Do not run `--update` on `strategy-manual` before
the pin is moved: it follows `c-borg-reference`, which no longer has the
directory. In the post-review refresh PR, repoint the pin to branch
`workbench/c-borg`, or retire S12 in favour of S14 (owner's call, still
open).

## 2026-11-10: source S14, the complete Angband Strategy Manual

The owner added neo-angband-tracker `docs/c-manual/` (default branch) as a
source. It is copied to `reference/c-manual/` by a new copy rule on the
`neo-angband-tracker` pin, moved to `777eea1` with `--update` (the C Borg
spec and plan copies are unchanged there). It is the finished version of
S12's draft: 17 chapters, appendices A1–A10, and the official manual. Chapter
15, *Notes for the borg*, is direct input for M2–M6 strategy and parameters.
S12 stays pinned until the owner retires it.

## 2026-11-10: M1 review findings (#39)

The M1 review (Kimi K3.1 Pro, on #39) had 4 blocking, 13 should-fix and 2
nit findings. They are answered in four PRs into `main`, each with a reply
on its comments:
- #42: RNG isolation of perception (`with_borg_rng`, hallucination
  regression in CI), and no loop without game time.
- #43: endpoint scoring (exact boundary, process receipts, per-class
  attempts, victory, pairing, zoo boundary by kind) and suite manifests.
- #44: death records (natural-death checks, family identity, index repair,
  attempt dedupe, registration on publication refusal, zoo deaths
  repatriated, CI death reconciliation).
- the sources PR: an atomic `--update`, and the zoo request (death
  evidence, boundary for every adapter, no stale fallback ref).

Rust play is unchanged by the fixes (full suite 45/45 identical), but #42
changed the Rust build inputs, so the Rust deaths are re-recorded at the new
source. The zoo change request was filed as borg-zoo#30 at the owner's
request; the issue needs the corrections of this PR (owner's call).

**After the re-review and the owner's merge of #39:** tag `m1`; amend
RULES 7 (owner decision 2026-11-10: sessions may open issues, not push,
on the family's repositories) and record borg-zoo#30 in this file and in
ZOO-CHANGE-REQUEST.md; then start M2 with a refresh PR (better-borg-c
D0015–D0016, borg-rs impact v7 engine re-sync as its own PR).

## 2026-11-09: two sources added (owner request)

- **S12, the Angband Strategy Manual:** neo-angband-tracker `docs/strategy/`
  on branch `c-borg-reference` (not the default branch), copied in
  `reference/strategy/` at `e67fd7d`. A new pin `strategy-manual` with two new
  pin fields: `branch` (the branch the refresh report and `--update` follow)
  and `vendor` (its own clone, `vendor/neo-angband-tracker-c-borg-reference`,
  so it never fights the default-branch clone over a checkout).
  `scripts/refresh-sources.py` and `scripts/setup.sh` support both; a new test
  covers a branch pin. The manual is a work in progress (chapter 3 missing)
  and moved twice on 2026-11-09: expect the refresh report to show it.
- **S13, the official command descriptions**
  (angband.readthedocs.io `command.html`): built from upstream
  `docs/command.rst`. Upstream `master` is at our engine pin `e473720`, so the
  page is `angband/docs/command.rst` in our tree; nothing copied.
- Not done: no strategy is taken from either yet. That is M3+ work; lifted
  knowledge cites the engine source, not the manual (docs/SOURCES.md S12).

## 2026-11-09, end of session: M1 under review

**State**
- M0 is reviewed (tag `m0`). **M1 is complete and in review:** PR #39
  (`main` → `reviewed`). All M1 issues (#3–#11) are closed. Do **not** start
  M2 (#12–#16) before the owner merges #39 (RULES 6).
- CI on `main` is green: the `rust` and `engine` jobs, with all four arms,
  controls against references, Rust determinism and exact boundary, and the
  tool tests.

**Waiting on the owner**
1. The M1 review on PR #39. Its body has the reviewer prompt.
2. Tag `m1` on the merge of #39. The zoo specimens should point at it.
3. Routing the zoo change request. The owner asked for a prompt for a
   borg-zoo session; it is the appendix of
   [ZOO-CHANGE-REQUEST.md](ZOO-CHANGE-REQUEST.md). The zoo work happens in
   borg-zoo, not here.
4. Whether the self-produced `harness/reference/smoke-c-bbc-m7` is enough as a
   regression reference.

**Next session, in order**
1. `git pull`, `bash scripts/setup.sh`, and `python3 scripts/refresh-sources.py`.
   Read new deaths first; better-borg-c moves daily (RULES 21).
2. If #39 has review findings: one fix PR into `main` per finding set, a
   reply on each comment, and the re-review. If it is merged: tag `m1` (if the
   owner has not), then start M2 with an `--update` refresh PR (PLAN: *How
   every milestone starts*), then #12 (T2.1 Observation).
3. Check whether borg-zoo has the reborn specimens and exact-boundary cells.
   If it does, the next comparison is G1, which comes after M5 (GOALS), so
   nothing to run yet.

**Local state (not in Git)**
- `build/` (all four arms, the GUI build `build/play-rust`, the death-workflow
  caches), `harness/out/` (all suite outputs, including `t18/`), `vendor/`
  (pinned clones) and the scratch play data. All of it can be regenerated
  with the commands in harness/README.md and AGENTS.md.


## 2026-11-09: T1.8 every-arm check, zoo compatibility and change request (#11) — M1 complete

- **Every arm** (`c`, `c-pristine`, `c-bbc-m7`, `rust`) × smoke and full:
  - no harness failure;
  - the exact-boundary contract holds (`harness/check-boundary.py`,
    192/192 runs);
  - a rerun gives byte-identical keys (192/192);
  - the oracles still hold (`c` = `smoke-c`, `c-pristine` = `full-pristine`
    45/45).
- **Zoo:** `harness/zoo-check.py` runs the pinned zoo's own borg-rs adapter
  (in-memory specimens, no zoo change). All three arms run on the zoo's
  smoke with no harness failure or config error. The gaps are survivors
  stopping after the cap and an empty `first_turn_at_depth`.
  [ZOO-CHANGE-REQUEST.md](ZOO-CHANGE-REQUEST.md) is **for the owner to
  route**: three catalogue specimens; the adapter's `-exact-turns`,
  boundary field and depth stream; optionally the endpoint in stats.
- Controls now build from a fresh checkout: the pinned sources are cloned on
  demand and missing blobs fetched in one batch (`c-bbc-m7` in 16 s). CI
  builds both controls, checks them against `smoke-pristine` and the new
  self-produced `smoke-c-bbc-m7`, and checks the Rust arm's exact boundary.
- The Rust arm's first full-suite deaths are recorded.

**Next:** the **M1 review PR** (`main` → `reviewed`). Do not start M2 before
the owner merges it (RULES 6).

## 2026-11-09: T1.7 endpoint calculator (#10)

- `tools/endpoint.py` computes the GOALS primary endpoint per arm and class:
  alive at T with max depth ≥ D, or a victory before T, over **every
  attempted run**. Operational outcomes are listed separately, with Wilson
  95% CIs and paired exact McNemar tests on shared (seed, class).
- Inputs are local suite outputs (runs listed in the suite but missing count
  as harness failures) and borg-zoo cells (filtered by specimen and profile).
  Survivors not observed at the exact boundary are refused unless
  `--allow-after-key`; zoo borg-rs-adapter cells are after-key until T1.8's
  request is done. Seven unit tests.
- **Demonstrations (descriptive, not claims):**
  - our full-exact suite at D=5, T=100k: `c` 2/45, `c-bbc-m7` 2/45,
    `c-pristine` 1/45;
  - zoo `classes-milestones` (observer, 1800 per specimen): pristine 3.0%
    [2.3, 3.9], bbc-m4 4.7% [3.8, 5.7]; Warrior 15.5% against 20.0%,
    Druid 0.5% against 0.0%. That is post hoc on existing data.

**Next:** #11 (T1.8: every-arm check including `rust`, zoo compatibility,
change request). Then the M1 review PR.

## 2026-11-09: T1.6 launchers, recording, replay, death tooling (#9)

- borg-rs's launchers and `tools/` are copied from `e5cb263`, with the same
  commands. The adaptations are listed at the top of `tools/README.md`:
  `.angband-reborn/`, build-input lists naming our crate, two tests on `HEAD`
  instead of `m5`, and the aggregator accepting our basic dossiers.
- `run.sh` calls the depth stream and the basic exporter only when `tools/`
  is present. The recording launchers run a frozen copy without it and record
  deaths in borg-rs's full format, so there is no double record.
- The Rust Borg now honours **any key stops it** (`Term_inkey`, as the C
  Borg's `user_abort`). Harness keys are unchanged (3/3 identical).
- **Checked:**
  - a headless recording (`run-borg-headless.py`) replays **VERIFIED**;
  - a GUI session under Xvfb (`run-rust-borg.sh`, Ctrl-Z, z, Escape stops,
    Ctrl-X saves and quits) records completely and replays **VERIFIED**;
  - borg-rs's 12 GUI-recording tests pass with that fixture;
  - the tool suite passes 87 tests (26 skip for lack of fixtures) and now
    runs in CI's engine job.
- `docs/deaths/` now has borg-rs's aggregated `index.json` and `README.md`,
  regenerated after every export.
- Observed during the GUI test: the placeholder Borg dies quickly (an
  apprentice on level 2), as expected until M3.

**Next:** #10 (T1.7, endpoint calculator), then #11 (T1.8).

## 2026-11-09: T1.5 engine boundary and minimal Borg (#8)

- `borg-reborn-rs/src/headers/` holds borg-rs's c2rust engine bindings,
  copied unchanged with attribution. `engine.rs` is our own small boundary:
  the player view, `map_info` terrain, the `^Z` command key, options, the
  `inkey_hack` hook, `do_cmd_borg`, and `borgh_decision` behind the `borgh`
  feature. It is the only module with `unsafe`.
- `borg.rs` is the control loop of ARCHITECTURE.md. Queued keys go first;
  rest and repeat get `EVT_NONE`; every action starts with `ESCAPE`; the
  Borg's own RNG is used and the game's never. On top of it sits a
  **placeholder** policy: take a down stair underfoot, rest when below 70%
  HP (never twice without time passing), else walk to a random neighbour
  not known to be a wall or shop. Six unit tests cover the policy.
- `^Z z` starts it, so borgh's start script and the GUI work unchanged. It
  sets `auto_more` and `easy_alter`.
- **Checked:** the Rust smoke suite runs with no harness failure, and two
  runs give byte-identical keys (3/3). The Elf Mage of seed 102 dies to a
  rock lizard at turn 743, and its dossier is recorded. CI now runs Rust
  smoke twice and compares.
- Not here: launchers (T1.6), and the Rust arm's exact-boundary check (T1.8).

## 2026-11-09: T1.4 run contract: exact boundary and depth stream (#7)

- **No engine-tree edit was needed.** borg-rs's borgh already stops at engine
  turn T from its per-turn hook (`-exact-turns`) and logs depth on every key.
  `run.sh` gains the flag `exact`. `tools/depth-stream.py` writes
  `depths.json` after every run (boundary rule, alive at stop, maximum depth
  at stop, first turn at each depth), and dossiers include it. There are new
  suites `smoke-exact` and `full-exact`; the existing suites keep their
  semantics and references.
- **Checked on every C arm** (`c`, `c-pristine`, `c-bbc-m7`), smoke and full
  (144 runs) with `harness/check-boundary.py`. Keys before the boundary are
  byte-identical to the normal runs, survivors stop exactly at the cap, and
  deaths before the cap are the same deaths. The `rust` arm gets the same
  check in T1.8.
- The `c` arm's first full-suite deaths are recorded, from a clean build.

**Next:** #10 (T1.7, endpoint calculator; needs only T1.4) and #8 (T1.5,
the minimal Borg).

## 2026-11-09: T1.3 control C Borgs (#6)

- `BORG_IMPL=c-pristine` and `c-bbc-m7` build our engine tree with a pinned C
  Borg overlaid in `build/<impl>.src/` (`harness/control-overlay.py`,
  `controls` in `pins.json`). The overlay is checked blob for blob, and the
  pinned source is part of the build-input hash.
- **`c-pristine` is byte-identical to borg-rs's pristine-C references** (the
  zoo's `4.2.6-rs-m0-c`): smoke 3/3 and full 45/45, in keys, messages and
  state. Our engine's only gameplay correction (#474) was not reached.
- **`c-bbc-m7`:** tag `4.2.6-impact-v7` changes nothing in the engine outside
  `src/borg/` (audited). It ran smoke and full with no harness failure. No
  byte oracle exists under borgh.
- **Deaths recorded (RULES 12):** 15 `c-pristine` and 16 `c-bbc-m7` deaths on
  the full suite are in `docs/deaths/`, unassessed. This is one run each, so
  it is descriptive, not a comparison (RULES 14).
- **Note for T1.8 (#11):** without the `menu` flag, borg-rs's full suite plays
  other classes for index 8; the zoo passes `-menu-letters`.
- Controls are not in CI yet: they need `vendor/` (scripts/setup.sh). The
  local commands are in harness/README.md.

**Next:** #7 (T1.4, run contract: exact boundary and depth stream; borgh
already has `opt_exact_turns`).

## 2026-11-09: REDESIGN-LESSONS.md read into the design

- better-borg-c wrote `docs/deaths/REDESIGN-LESSONS.md` for this repository:
  thirteen lessons from D0001–D0014 and four owner manual takeovers. It came
  in with refresh #31, together with death D0014.
- [LESSONS-TRACE.md](LESSONS-TRACE.md) maps each lesson to our principle,
  our architecture section and the task that builds it.
- GOALS principles 4–8 are extended, and principle 14 is new (equipment by
  what it does). D3 now states that the threat model uses full race data
  (spoiler knowledge), as the C Borg does.
- ARCHITECTURE adds monster-track confidence, one assessment and tolerance
  for every consumer, cache validity, the escape ladder, consumable value
  and destination-aware level changes.
- PLAN adds D0014 to the fixtures, and owner manual branches as scenario
  tests.
- The refresh report now names new and changed registry documents.

**Next:** #6 (T1.3, control C Borgs).

## 2026-11-09: T1.2 harness, build identity, death export (#5)

- `harness/` is borg-rs's at `e5cb263`, with three adaptations listed in
  [harness/README.md](../harness/README.md):
  - `identity.sh` names our crates;
  - `run.sh` ends with `tools/export-death.py`;
  - `measurement/rust-overlay.py` copies both workspace members.
- `BORG_IMPL=rust` links through the seam package `borg-rs/` (the name the
  imported CMake builds), which re-exports `borg-reborn-rs`. The engine
  boundary module `engine` has a no-op `do_cmd_borg` until T1.5.
- **Import proved:** the `c` smoke suite is byte-identical (keys, messages,
  state) to borg-rs's own C build at `e5cb263`. That output is
  `harness/reference/smoke-c`. borg-rs's committed smoke reference is the
  pristine-C one (unchanged since `aa4ad6c`) and is kept as
  `harness/reference/smoke-pristine` for T1.3.
- **First death:** smoke-mage-2 (seed 102, Elf Mage, `c` arm) dies to an
  acolyte at turn 7195. It is the same as in borg-rs. Its dossier is our first
  case in `docs/deaths/`.
- CI gains an `engine` job: build `c`, run smoke, diff against `smoke-c`,
  build `rust`, check that the binary reads as `rust`.

**Next:** #6 (T1.3, control C Borgs) and #7 (T1.4, which needs T1.3).

## 2026-11-09: T1.1 engine tree imported (#4)

- `angband/` is borg-rs `e5cb263` (impact v6) `angband/`, split with
  `git subtree --ignore-joins` into 29 commits and added unsquashed. The tree id
  `5cc1b1c…` is recorded in `pins.json` (`engine_tree`), and a repository test
  pins it.
- [ENGINE-TREE.md](ENGINE-TREE.md) lists what differs from upstream
  `e473720`:
  - 8 added files (borgh, recording);
  - 14 engine files, all behind `BORG_HARNESS`/`BORG_RECORDING` except
    CMake, `main.h` declarations and the #474 cure-all correction;
  - 26 C Borg files (borg-rs's adopted milestones 2–6).
- It also gives the re-sync procedure.
- borg-rs's `FORK-CORRECTIONS.md` and `adoption/status.json` are copied to
  `reference/borg-rs/` by a new copy rule. The refresh report flags when
  borg-rs's `angband/` tree moves.
- Nothing is built yet: building and the harness are T1.2 (#5).

**Next:** #5 (T1.2: harness, build identity, death export before the first
game).

## 2026-11-09: new deaths read into the evidence

better-borg-c D0011–D0013 (from the first refresh) are folded in. SOURCES
has three new death patterns (fear by list, unknown items left unknown,
wrong kit valuation) and extended rows. GOALS principles 4–8 cite them, and
PLAN's death fixtures include them. The research digest
(`docs/research/DEATHS-AND-ISSUES.md`) is a dated snapshot of 2026-11-08;
`reference/deaths/INDEX.md` is the live list.

**Next:** #4 (T1.1, import the engine tree from borg-rs at `e5cb263`).

## 2026-11-09: T1.0 source refresh routine (#3)

- `scripts/refresh-sources.py` reports, per pin: commits since the pin, new
  and changed death cases, new tags (new control arms), changed C Borg
  functions we cite, and issues updated since the pin was read. `--update`
  moves the pins, refreshes `reference/` (copy rules in `pins.json`) and
  regenerates `reference/deaths/INDEX.md`. Tests on synthetic repositories
  cover the report, update, dirty vendor and offline cases; CI runs them.
- RULES 21–23: every session starts with the report, every milestone with an
  `--update` PR; baselines are exact; lifted knowledge is watched.
- **First `--update` (PR closing #3):** borg-rs `6b58a46`→`e5cb263` (impact v6; its C Borg now differs from upstream in 26 files, issue #4 updated), better-borg-c `63fbc1f`→`8b26042` (deaths D0011–D0013 copied), borg-zoo `ffe5a05`→`9cd7423`, borg-agent `697abde`→`8f1dd7f`. The new deaths are read into SOURCES/GOALS/PLAN in a separate PR (RULES 21).
- **First real report (2026-11-09):** better-borg-c has three new deaths
  (D0011–D0013), 25 commits and 75 updated issues; borg-rs moved 7 commits,
  borg-zoo 3, borg-agent 9. The `--update` PR that brings them in is the
  last step of #3.

## 2026-11-09: M0 reviewed, M1 started, backlog moved to issues

- The owner merged the M0 review (#1); tag `m0` is on the merge
  (`ae64f12`). D1–D11 are accepted (GOALS.md).
- The backlog is now GitHub issues (owner request): GitHub milestones
  M1–M7+, labels `task`, `process`, `in progress` and `owner`, and issues
  #3–#24 for T1.0–T3.8, each with a **Needs** line linking its
  dependencies. PLAN.md keeps only milestones, exits and method.
- **Next:** #3 (T1.0, the source-refresh routine over every pin, death
  reports included; owner request), then #4 (T1.1, import the engine tree
  at refreshed pins).

## 2026-11-08: M0 review findings addressed (T0.4)

The owner's review of PR #1 had six findings, all verified and addressed
in the T0.4 PR:

| Finding | Change |
| --- | --- |
| Blocking: the primary endpoint has no tooling that computes it. borgh overshoots the cap, the adapter leaves `first_turn_at_depth` empty, and zoo stats exclude operational outcomes | GOALS *Observation and scoring rule*; PLAN T1.4 (exact boundary, depth stream), T1.7 (`tools/endpoint.py`), T1.8 (zoo change request); G1 prerequisites stated |
| Blocking: the imported `BORG_IMPL=c` is borg-rs's adopted C Borg (22 files differ from upstream), and observer-harness zoo cells cannot be paired with borgh | PLAN T1.1 corrected; new T1.3: `c-pristine` and `c-bbc-m7` controls built here on the same engine and `borgh`, `c-pristine` checked against the zoo's `4.2.6-rs-m0-c`; GOALS D2/D8/G1/G2 |
| setup `--strict` did not check that checkouts were at the pins | `scripts/setup.sh` checks out pins, reports WRONG REVISION, never moves dirty checkouts, `--checkout` moves clean ones (all three paths tested) |
| Death copies missed `overall-assessment.md` and other small evidence files | All root-level `.md`/`.json` of each case copied; external links listed in `reference/README.md`; a test fails on dangling internal links (checked that it fails when an assessment is removed) |
| Death export came after the first playing run | Minimal death export is part of T1.2, before its first smoke run; RULES 12 applies from the first game. Follow-up review: T1.4 checks only the C arms, and the every-arm check (`rust` included) is in T1.8 and the M1 exit |
| Nit: `BORG_IMPL=reborn` | `rust` everywhere, as borg-rs |

Follow-up review on #2: borg-rs is reclassified everywhere as engine and
tooling source and perception oracle, not a control. Its zoo results are
context (GOALS recommendation, non-goals and D8; AGENTS).

Still open for the owner: D1–D11, and routing the zoo change request once
T1.8 writes it.

## 2026-11-08: milestone 0 (foundation), ready for review

**Done**
- T0.1: sources read and evaluated ([SOURCES.md](SOURCES.md)). Specification,
  plan, curated death dossiers and zoo catalogue copied to `reference/`
  with provenance. Prior art outside Angband surveyed (AutoAscend, BotHack,
  TAEB/Saiph, qw, Rog-O-Matic, NetPlay). A digest of deaths and issues
  by subsystem is in [research/DEATHS-AND-ISSUES.md](research/DEATHS-AND-ISSUES.md).
- T0.2: goals, principles, staged targets and eleven owner decisions,
  all *proposed* ([GOALS.md](GOALS.md)). The design direction is in
  [ARCHITECTURE.md](ARCHITECTURE.md).
- T0.3: [PLAN.md](../PLAN.md) (M0–M7, tasks for M1–M3),
  [RULES.md](../RULES.md), [AGENTS.md](../AGENTS.md) (model-agnostic entry;
  `CLAUDE.md` only includes it), `pins.json`, `scripts/setup.sh`, the
  empty crate `borg-reborn-rs/` (`staticlib` + `rlib`, as borg-rs),
  repository consistency tests, CI.

**For the owner to decide (milestone 0 review)**
- D1–D11 in [GOALS.md](GOALS.md#owner-decisions). The ones that shape M1
  most: D2 (engine tree from borg-rs), D3 (information level), D4 (reuse
  of borg-rs code), D9 (workflow).
- Whether the `reviewed` branch starts at the root commit (so the M0 review
  PR shows everything), as set up here.

**Next**
- Done on 2026-11-09: see the newest entry above.

**Known limits**
- Nothing is built against the engine yet. The CI checks only the empty
  crate and the documents.
- The death digest was produced with a model's help from the issue bodies
  and dossiers at the pinned commits. Its classifications are leads, not
  findings. SOURCES.md's table of death patterns was checked by hand
  against the dossiers' `assessment.json`.
