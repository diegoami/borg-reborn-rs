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
| **M1. The seam** | The engine and harness from borg-rs; the reborn crate linked behind `inkey_hack` as `BORG_IMPL=rust`; a minimal Borg that starts, perceives and acts safely; same launchers | `harness/build.sh` builds both `c` and `rust`; a seed run twice gives identical `keys.log`; the borg-rs smoke suite runs end to end on the reborn Borg with no harness failure; launchers start a GUI game with Ctrl-Z, z | not started |
| **M2. Perception and world model** | `Observation`, world model, knowledge tables, decision trace, parameter table | Perception agrees with borg-rs's state on recorded runs (declared exceptions only); observations serialise and replay into the world model without the game; every engine read is listed (D3 audit) | not started |
| **M3. A Borg that plays** | Executor and prompts, paths, exploration, melee, pickup, stairs, rest, town restock; the first strategy without the full tactical layer; death dossiers | **G0**: no stall, crash or harness failure on the smoke suites; deterministic replay; every natural death recorded with its decision trace | not started |
| **M4. Danger and survival** | The threat model and its calibration, reserves, escapes, the tactical layer, survival invariants | Calibration report on recorded runs; fixtures from the death cases (below) pass; early-suite deaths measured against the C Borg (descriptive) | not started |
| **M5. Readiness and progression** | Readiness policy per class, equipment valuation, item use, spells for casters, ranged combat | **G1** on `classes-1800` (owner starts the zoo run) | not started |
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
- **Our own deaths** are recorded the same way (RULES 12), and each
  milestone review lists them with causes.
- **Population numbers** come from borg-zoo. Locally we run identity and
  smoke suites only.

## The tasks

### Status

| Task | Milestone | Status | Needs |
| --- | --- | --- | --- |
| T0.1 Research and sources | M0 | done | — |
| T0.2 Goals and owner decisions | M0 | done (decisions proposed) | T0.1 |
| T0.3 Plan, rules, agent entry, crate skeleton, CI | M0 | done | T0.2 |
| T1.1 Import the engine tree | M1 | todo | M0 review |
| T1.2 Harness and build identity | M1 | todo | T1.1 |
| T1.3 Engine boundary and the minimal Borg | M1 | todo | T1.2 |
| T1.4 Launchers, recording, replay | M1 | todo | T1.3 |
| T1.5 Zoo compatibility check | M1 | todo | T1.3 |
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
| T3.6 Death dossiers with decision traces | M3 | todo | T3.4 |
| T3.7 Smoke and stall suites, G0 report | M3 | todo | T3.4–T3.6 |

M4–M7 are broken into tasks at the M3 review, from what M1–M3 teach.

### T1.1 Import the engine tree

Bring borg-rs's `angband/` directory into this repository **with its
history** (git subtree from borg-rs at a pinned commit, recorded in
`pins.json`). It is upstream `e473720` plus the `borgh` harness hooks
and the fork corrections borg-rs declares in `docs/FORK-CORRECTIONS.md`.
Copy that file and say which corrections are in. Change nothing in the
engine. The C Borg (`angband/src/borg/`) comes with it and stays buildable
as `BORG_IMPL=c`, so the original Borg is always available as an in-tree
baseline.

### T1.2 Harness and build identity

Copy borg-rs's `harness/` scripts (`build.sh`, `run.sh`, `suite.sh`,
`identity.sh`, `diff`, `supervise.py`, `suites/smoke`, `borg.txt`) with
the same names, arguments and outputs (`keys.log`, `summary.json`,
`BUILD_ID`). `BORG_IMPL=rust` builds **this** crate. Keep borg-rs's
reference logs out unless a test needs them. Exit: `BORG_IMPL=c` reproduces
borg-rs's C smoke references byte for byte, which proves the import.

### T1.3 Engine boundary and the minimal Borg

Bring `engine.rs` over from borg-rs, narrowed to what is used, with
attribution. Implement the control loop of ARCHITECTURE.md (queue,
`EVT_NONE` pass-through, own RNG, prompt catching for `-more-` and
yes/no). The minimal Borg reads its HP and position and does something
harmless and progressing: walk, take stairs when on them, rest when hurt.
Exit: two runs of the same seed give identical `keys.log`; no harness
failure on `smoke`.

### T1.4 Launchers, recording, replay

`run-rust-borg.sh`, `start-borg.sh`, `run-borg-headless.py`,
`replay-borg.py`, `produce_death.py`, `replay_next_death.py`, `tools/`:
same commands and options as borg-rs. Save directory `.angband-reborn/`
so it does not collide with borg-rs's `.angband-rust/`. Exit: GUI start
with Ctrl-Z, z; a headless recording replays with `--verify-only`.

### T1.5 Zoo compatibility check

Run borg-zoo's borg-rs adapter (`zoo/borgrs.py`, at the pinned zoo commit)
against this repository locally, on the smoke suite, without changing the
zoo. Report what works and what an adapter change would need, for the
owner to route to borg-zoo (RULES 8).

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
