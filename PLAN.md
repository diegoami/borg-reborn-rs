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
**owner merges** the review PR (or tells a session to), which advances
`reviewed`, and an annotated tag `mN` is recorded on the merge. The findings
become issues in the next milestone.

Since 2026-11-09 the plan follows docs/SPEC.md (§10): milestone E, our own
integration of upstream Angband, comes before everything else. M0 and M1 stay
as history; M1's engine tree and harness came from borg-rs and are replaced in E.

| Milestone | What it delivers | Exit (all must hold) | State |
| --- | --- | --- | --- |
| **M0. Foundation** | Research, goals, plan, repository conventions, reference sources, an empty crate that builds | Owner confirms or amends D1–D11 in [GOALS.md](docs/GOALS.md) | **done**: reviewed in #1, tag `m0` (2026-11-09) |
| **M1. The seam and the yardstick** | The engine and harness from borg-rs; the reborn crate linked behind `inkey_hack` as `BORG_IMPL=rust`; same-frontend control C Borgs; the run contract (exact boundary, depth stream); death export before the first playing run; a minimal Borg; same launchers; the endpoint calculator | `harness/build.sh` builds `c`, `c-pristine`, `c-bbc-m7` and `rust`; `c-pristine` reproduces borg-rs's pristine-C references (T1.3); a seed run twice gives identical `keys.log`; on every arm, `rust` included, the smoke suite runs end to end with no harness failure and the run-contract check holds (T1.8); every natural death has a dossier, from the first game played; launchers start a GUI game with Ctrl-Z, z; the zoo change request is with the owner (T1.8) | **done**: reviewed in #39, tag `m1` (2026-11-09) |
| **E. Our own integration** | docs/SPEC.md §3–§4: upstream Angband byte for byte, our patch series, our headless frontend, generated bindings, our build and harness, only our Borg, the zoo contract written down and checked; the removals of SPEC.md §8 and the documents of §9 | SPEC.md §10, all eight points | **done**: merged in #92, tag `e` (2026-11-09), **merged before review**; reviewed after the fact on #92 (2026-11-10), findings fixed in #94–#96 |
| **M2. Perception and world model** | `Observation`, world model, knowledge tables, decision trace, parameter table | Perception agrees with the engine on recorded runs (declared exceptions only; SPEC.md §7); observations serialise and replay into the world model without the game; every engine read is listed (D3 audit, SPEC.md §5) | **complete**: T2.1–T2.5 done (#60–#63, T2.5 in the PR closing #16); review PR next |
| **M3. A Borg that plays** | Executor and prompts, paths, exploration, melee, pickup, stairs, rest, town restock; the first strategy without the full tactical layer; death dossiers | **G0**: no stall, crash or harness failure on the smoke suites; deterministic replay; every natural death recorded with its decision trace | not started |
| **M4. Danger and survival** | The threat model and its calibration, reserves, escapes, the tactical layer, survival invariants | Calibration report on recorded runs; fixtures from the death cases (below) pass; early-suite deaths measured against the C Borg (descriptive) | not started |
| **M5. Readiness and progression** | Readiness policy per class, equipment valuation, item use, spells for casters, ranged combat | **G1** on `classes-1800` against the same-frontend controls (owner starts the zoo run; prerequisites in [GOALS.md](docs/GOALS.md#observation-and-scoring-rule)) | not started |
| **M6. The family** | Class profiles, the first class-specific strategy where M5's data shows the need, a tuning loop (screening and racing) | **G2**, confirmed on fresh seeds | not started |
| **M7+** | Deeper suites, mid game, death-driven iterations | **G3**, then G4 | not started |

## How every milestone starts

With a source refresh (RULES 21): `scripts/refresh-sources.py --update` in
its own PR, then reading what moved, new deaths first. New better-borg-c
milestones become new control arms, and changed C Borg functions we cite
are reviewed.

## How every milestone from M3 on is judged

- **Fixtures from deaths.** Each death case in
  [`reference/deaths/`](reference/deaths/) whose situation can be
  rebuilt becomes a decision fixture: a recorded or constructed
  `Observation` and the decision it must *not* make (drink the cure, shoot
  the kobold, leave the mold). Seed cases: bbc D0002 and rs D0005
  (unused cure), bbc D0010 (shoot instead of flee), bbc D0008 and rs D0006
  (oscillation), bbc D0009 (food against a pickup loop), bbc D0006
  (Deep Descent against readiness), bbc D0011 (drink the known CLW; no
  Deep Descent from depth 4 at CL10; wield the Maul), bbc D0012 (attack
  the 2-HP mold), bbc D0013 (step back from the floating eye instead of
  reading the only Phase Door; do not flee a level over one small kobold),
  bbc D0014 (drink Speed and CLW against the blood falcon; keep the last
  Phase Door at full HP; no Deep Descent from depth 2).
  New deaths from the source refresh (RULES 21) add cases.
- **Owner manual branches as scenario tests** (REDESIGN-LESSONS L13): the
  `owner-manual-branch.json` of bbc D0010, D0012, D0013 and D0014 gives a
  verified prefix and a known-feasible alternative. Starting from the same
  state, our decision must be at least as good. The full lesson-to-design
  map is [docs/LESSONS-TRACE.md](docs/LESSONS-TRACE.md).
- **Our own deaths** are recorded from the first game played in M1
  (T1.2's death export, RULES 12), whichever arm died, and each milestone
  review lists them with causes.
- **Population numbers** come from borg-zoo. Locally we run identity and
  smoke suites only.

## The tasks

**Tasks are GitHub issues** (since the M0 review, 2026-11-09): one issue
per task, labelled `task`, in the GitHub milestone of its PLAN milestone,
with a **Needs** line linking the issues it depends on. The issue body is
the task's specification. This file keeps the milestones, their exit
criteria and the method; it no longer lists tasks.

- [M1 tasks](https://github.com/diegoami/borg-reborn-rs/milestone/1): #3 (T1.0, source refresh) to #11 (T1.8)
- [E tasks](https://github.com/diegoami/borg-reborn-rs/milestone/8): #71 (TE.1) to #79 (TE.9), from docs/SPEC.md §10
- [M2 tasks](https://github.com/diegoami/borg-reborn-rs/milestone/2): #12 to #16 (all done)
- [M3 tasks](https://github.com/diegoami/borg-reborn-rs/milestone/3): #17 to #24 (#24 breaks M4–M7 into tasks)

```sh
gh issue list --milestone "E. Our own integration" --state open
```

A finding of a milestone review becomes an issue in the next milestone. A
new task found during work is a new issue, filed in the current milestone
only if the milestone cannot pass its exit without it; otherwise it goes in a
later milestone and the handover says so.

M0's tasks (T0.1–T0.4) were done in this file's earlier versions and in
PRs #1 and #2; tag `m0`.

## After M7 (not planned yet)

- A "fair-play" information profile (screen only), measured separately (D3).
- Learned components for narrow decisions, from the decision trace.
- Neo Angband, through borg-agent's or Mandos's adapters.
