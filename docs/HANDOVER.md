# Handover

*Where the work stands, for the next session. Update it in every PR.*

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
