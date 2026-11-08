# Plan

*Step 3 of the initialisation. The goals are in
[docs/GOALS.md](docs/GOALS.md) and the design direction in
[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md). Where this plan and the goals
differ, the goals win. Say so in your PR and propose a fix to whichever
is wrong.*

## Milestones, and when we stop for a review

We work in spurts. A **milestone** is a named set of tasks. Inside a
milestone a session works normally: one task per branch and PR, and it
**merges its own PR into `main` once CI is green** (RULES 1).

**When the last task of a milestone is merged, the session stops.** It does
not start the next milestone. It opens the **review PR** instead:

```sh
gh pr create --base reviewed --head main --title "Milestone N review: <what it delivers>"
```

`reviewed` is a long-lived branch at the last reviewed state of `main`, so
the review PR's diff is exactly the milestone's work. Its body lists the
tasks, what each verified, what was not checked and the open questions,
and carries the prompt a reviewer needs. A reviewer (the owner, or a model
of another family than the one that wrote the code) posts findings. The
**owner merges** the review PR, which advances `reviewed`, and records an
annotated tag `mN` on the merge. The findings become the first task of the
next milestone.

| Milestone | What it delivers | Exit (all must hold) | State |
| --- | --- | --- | --- |
| **M0. Foundation** | Research, goals, plan, repository conventions, reference sources, an empty crate that builds | Owner confirms or amends D1–D11 in [GOALS.md](docs/GOALS.md) | **in review** |
| **M1. The seam and the yardstick** | The engine and harness from borg-rs; the reborn crate linked behind `inkey_hack` as `BORG_IMPL=rust`; same-frontend control C Borgs; the run contract (exact boundary, depth stream); death export before the first playing run; a minimal Borg; same launchers; the endpoint calculator | `harness/build.sh` builds `c`, `c-pristine`, `c-bbc-m7` and `rust`; `c-pristine` reproduces borg-rs's pristine-C references (T1.3); a seed run twice gives identical `keys.log`; the smoke suite runs end to end on every arm with no harness failure; every natural death has a dossier; launchers start a GUI game with Ctrl-Z, z; the zoo change request is with the owner (T1.8) | not started |
| **M2. Perception and world model** | `Observation`, world model, knowledge tables, decision trace, parameter table | Perception agrees with borg-rs's state on recorded runs (declared exceptions only); observations serialise and replay into the world model without the game; every engine read is listed (D3 audit) | not started |
| **M3. A Borg that plays** | Executor and prompts, paths, exploration, melee, pickup, stairs, rest, town restock; the first strategy without the full tactical layer; death dossiers | **G0**: no stall, crash or harness failure on the smoke suites; deterministic replay; every natural death recorded with its decision trace | not started |
| **M4. Danger and survival** | The threat model and its calibration, reserves, escapes, the tactical layer, survival invariants | Calibration report on recorded runs; fixtures from the death cases (below) pass; early-suite deaths measured against the C Borg (descriptive) | not started |
| **M5. Readiness and progression** | Readiness policy per class, equipment valuation, item use, spells for casters, ranged combat | **G1** on `classes-1800` against the same-frontend controls (owner starts the zoo run; prerequisites in [GOALS.md](docs/GOALS.md#observation-and-scoring-rule)) | not started |
| **M6. The family** | Class profiles, the first class-specific strategy where M5's data shows the need, a tuning loop (screening and racing) | **G2**, confirmed on fresh seeds | not started |
| **M7+** | Deeper suites, mid game, death-driven iterations | **G3**, then G4 | not started |

## How every milestone from M3 on is judged

- **Fixtures from deaths.** Each death case in
  [`reference/deaths/`](reference/deaths/) whose situation can be
  rebuilt becomes a decision fixture: a recorded or constructed
  `Observation` and the decision it must *not* make (drink the cure, shoot
  the kobold, leave the mold). Seed cases: bbc D0002 and rs D0005
  (unused cure), bbc D0010 (shoot instead of flee), bbc D0008 and rs D0006
  (oscillation), bbc D0009 (food against a pickup loop), bbc D0006
  (Deep Descent against readiness).
- **Our own deaths** are recorded from the first playing run in M1
  (T1.5, RULES 12), and each milestone review lists them with causes.
- **Population numbers** come from borg-zoo. Locally we run identity and
  smoke suites only.

## The tasks

### Status

| Task | Milestone | Status | Needs |
| --- | --- | --- | --- |
| T0.1 Research and sources | M0 | done | — |
| T0.2 Goals and owner decisions | M0 | done (decisions proposed) | T0.1 |
| T0.3 Plan, rules, agent entry, crate skeleton, CI | M0 | done | T0.2 |
| T0.4 M0 review findings (PR #1) | M0 | done | T0.3 |
| T1.1 Import the engine tree | M1 | todo | M0 review |
| T1.2 Harness and build identity | M1 | todo | T1.1 |
| T1.3 Control C Borgs on the same frontend | M1 | todo | T1.2 |
| T1.4 Run contract: exact boundary and depth stream | M1 | todo | T1.2 |
| T1.5 Engine boundary, minimal Borg, death export | M1 | todo | T1.4 |
| T1.6 Launchers, recording, replay, death tooling | M1 | todo | T1.5 |
| T1.7 Endpoint calculator | M1 | todo | T1.4 |
| T1.8 Zoo compatibility and change request | M1 | todo | T1.3, T1.5, T1.7 |
| T2.1 Observation | M2 | todo | M1 |
| T2.2 World model | M2 | todo | T2.1 |
| T2.3 Knowledge tables | M2 | todo | T2.1 |
| T2.4 Decision trace and parameter table | M2 | todo | T2.1 |
| T2.5 Perception oracle against borg-rs | M2 | todo | T2.2 |
| T3.1 Executor and prompts | M3 | todo | M2 |
| T3.2 Paths and exploration | M3 | todo | T3.1 |
| T3.3 First strategy: modes, goals, budgets | M3 | todo | T3.2 |
| T3.4 Melee, pickup, stairs, rest | M3 | todo | T3.3 |
| T3.5 Town: sell, buy reserves | M3 | todo | T3.3 |
| T3.6 Decision traces in death dossiers | M3 | todo | T3.4 |
| T3.7 Smoke and stall suites, G0 report | M3 | todo | T3.4–T3.6 |

M4–M7 are broken into tasks at the M3 review, from what M1–M3 teach.

### T1.1 Import the engine tree

Bring borg-rs's `angband/` directory into this repository **with its
history** (git subtree from borg-rs at the pinned commit in `pins.json`).
It is upstream `e473720` plus the `borgh` harness hooks, the fork
corrections borg-rs declares in `docs/FORK-CORRECTIONS.md`, and **the C
Borg fixes borg-rs has adopted**: at the pin, 22 files under
`angband/src/borg/` differ from upstream (impact-v5 accepted, see
borg-rs `docs/adoption/status.json`). So the tree's own C Borg is
borg-rs's adopted C Borg. It is **neither** the original C Borg **nor**
better-borg-c milestone 7, and it is never used as a baseline (T1.3
builds those). Copy `FORK-CORRECTIONS.md`, list the corrections and the
adopted C carries that are in the tree, and change nothing in the engine.

### T1.2 Harness and build identity

Copy borg-rs's `harness/` scripts (`build.sh`, `run.sh`, `suite.sh`,
`identity.sh`, `diff`, `supervise.py`, `suites/smoke`, `borg.txt`) with
the same names, arguments and outputs (`keys.log`, `summary.json`,
`BUILD_ID`, `build/<impl>/`). `BORG_IMPL=rust` builds **this** crate.
`BORG_IMPL=c` builds the tree's own C Borg. Keep borg-rs's reference logs
out unless a test needs them. Exit: `BORG_IMPL=c` reproduces borg-rs's C
smoke references at the pinned borg-rs commit byte for byte, which proves
the import.

### T1.3 Control C Borgs on the same frontend

The baselines (GOALS D8) must run on **our** engine tree, through **our**
`borgh`, under the same run contract as the reborn Borg. Results from
the better-borg-c observer cannot be paired with borgh runs: same-seed
equivalence between the two harnesses is unverified (borg-zoo
`docs/DESIGN.md`, *Limits*). `harness/build.sh` gains two
implementations, each compiling a pinned C Borg source instead of the
tree's own `src/borg/`, overlaid in the build directory (the tree is not
edited):

| `BORG_IMPL` | C Borg source | Check |
| --- | --- | --- |
| `c-pristine` | upstream `e473720` `src/borg/` | Same keys as borg-rs's pristine C under borgh, the zoo's `4.2.6-rs-m0-c` (borg-rs `aa4ad6c`, impl `c`), on the borg-rs smoke and full suites. A difference must be attributed to a declared engine correction in our tree (T1.1), or it is a defect |
| `c-bbc-m7` | better-borg-c tag `4.2.6-impact-v7` (`3573d74`) `src/borg/` | The overlay equals the tag's `src/borg/` blob for blob. better-borg-c's engine-side carries up to milestone 7 are listed as already in our tree, applied in the overlay, or declared as a difference. It builds and plays the smoke suite with no harness failure. No byte-identity oracle exists yet (none runs bbc-m7 under borgh) |

`BUILD_ID` records the overlay's source commit and hash. The zoo's borg-rs
adapter passes `BORG_IMPL` through unchanged and reads `build/<impl>/`, so
these arms need only catalogue entries in borg-zoo (T1.8), not adapter
code. A later better-borg-c milestone becomes a new arm (`c-bbc-m8`, …),
and an existing arm is never redefined.

### T1.4 Run contract: exact boundary and depth stream

The Tunable Borg Plan (*Observation and scoring rule*) makes this a
prerequisite of any shared measurement. borg-rs's `borgh` checks its cap
only after a returned key, so a 100,000-turn cap has ended at 101,191.
Add to our `borgh`, as harness-only hooks under `#ifdef BORG_HARNESS`:

- **Exact boundary:** stop at engine turn T itself, after the world-turn
  increment that reaches T. Alive, depth and maximum depth are observed
  there, and `summary.json` says so (`boundary: "exact"`, `game_turns ==
  T`).
- **Depth stream:** one line per change of depth (engine turn, depth,
  maximum depth, character level, HP), giving the first engine turn at
  each depth.

Exit: for every arm, the keys before the boundary are byte-identical to an
unmodified-`borgh` run of the same seed (so the hooks do not change play),
and the boundary and the stream are checked on seeds that die before T,
survive to T, and change depth.

### T1.5 Engine boundary, minimal Borg, death export

Bring `engine.rs` over from borg-rs, narrowed to what is used, with
attribution. Implement the control loop of ARCHITECTURE.md (queue,
`EVT_NONE` pass-through, own RNG, prompt catching for `-more-` and
yes/no). The minimal Borg reads its HP and position and does something
harmless and progressing: walk, take stairs when on them, rest when hurt.

**Before the first playing run**, add minimal death export: a natural death
in any harness run writes a basic dossier under `docs/deaths/` in borg-rs's
`record.json` format (case id, seed, build identity, implementation,
summary, `keys.log` hash, terminal state, cause), plus an `assessment.json`
with status "unassessed". T1.6 upgrades basic dossiers in place to the full
format, and T3.6 adds the decision trace. Exit: two runs of the same seed
give identical `keys.log`; no harness failure on `smoke`; a run that dies
produces a dossier.

### T1.6 Launchers, recording, replay, death tooling

`run-rust-borg.sh`, `start-borg.sh`, `run-borg-headless.py`,
`replay-borg.py`, `produce_death.py`, `replay_next_death.py`,
`tools/record-death.py`, `tools/aggregate-deaths.py` and the rest of
`tools/`: same commands and options as borg-rs. Save directory
`.angband-reborn/`, so it does not collide with borg-rs's `.angband-rust/`.
Exit: GUI start with Ctrl-Z, z; a headless recording replays with
`--verify-only`; the dossiers from T1.5 are upgraded and aggregated.

### T1.7 Endpoint calculator

`tools/endpoint.py` computes the primary endpoint of GOALS.md. It runs
over our suite outputs and over borg-zoo result cells (the public
`results` branch): per class and per arm, the share of **all attempted
runs** that are alive at the exact boundary having reached D (or won
before T). Operational outcomes are reported separately and counted as
non-successes. It also gives Wilson intervals and paired McNemar
comparisons on shared seeds. This exists because the zoo's own `goal_rate`
divides by `death`/`turn_cap`/`completion` cells only (`zoo/stats.py` at
the pin), which is not this endpoint. Exit: unit tests on synthetic cells
covering every outcome kind; a run on real smoke outputs.

### T1.8 Zoo compatibility and change request

Run borg-zoo's borg-rs adapter (`zoo/borgrs.py`, at the pinned zoo commit)
locally against this repository for `rust`, `c-pristine` and `c-bbc-m7`
on the smoke suite, without changing the zoo. Then write the change
request for the owner to route to borg-zoo (RULES 7):

- catalogue specimens for the three arms;
- the adapter reading the depth stream into `first_turn_at_depth` and
  recording the boundary rule;
- optionally, a stats option for the all-attempted denominator and the
  per-class composite endpoint. Without it, T1.7 computes them from the
  cells.

G1 cannot be claimed until the specimens exist in the zoo catalogue and
T1.4 and T1.7 are done.

### T2.1–T2.5

See ARCHITECTURE.md L1–L3 and Cross-cutting. T2.5 is the perception
oracle: on the same recorded runs, the `Observation`/world model agrees
with borg-rs's `BorgState` after update, field by field, with each
difference either fixed or declared (and justified by spec §11 or §17).

### T3.1–T3.7

See ARCHITECTURE.md L4–L6. M3 does not need the full threat model: a
conservative placeholder (the C Borg's danger sum, lifted with citations)
is acceptable until M4 replaces it. G0 is about playing without getting
stuck, not about surviving well.

## After M7 (not planned yet)

- A "fair-play" information profile (screen only), measured separately (D3).
- Learned components for narrow decisions, from the decision trace.
- Neo Angband, through borg-agent's or Mandos's adapters.
