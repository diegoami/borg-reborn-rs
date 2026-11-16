# Plan

*Step 3 of the initialisation. The goals are in
[docs/GOALS.md](docs/GOALS.md) and the design direction in
[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md). Where this plan and the goals
differ, the goals win. Say so in your PR and propose a fix to whichever
is wrong.*

Track R current state: [R4 handover](docs/R431-HANDOVER.md) and [work plan](docs/R4-PLAN.md); historical R3 [scoped handover](docs/R366-HANDOVER.md) and
[combined local validation report](docs/R3-EXIT-REPORT.md). New canonical
validation evidence belongs in borg-cemetery; source receipts pin its bytes.

## Milestones, and when we stop for a review

**Track P exception (owner, 2026-11-15):** task PRs target `track-p` and
are merged there by the session after green CI. Every milestone stops for
an owner-chosen review on a pinned `track-p` → `main` PR. Review fixes
also land in `track-p`; only owner acceptance plus green CI permits
promotion to `main`, merged by the owner or expressly delegated. Do not
start the next P milestone or carry unreviewed P work onto main through
another track. CI is a compatibility gate, not independent review. The
following `main` → `reviewed` description is the historical/default flow
for tracks without their own exception; it does not apply to new P work.

We work in spurts. A **milestone** is a named set of tasks. Inside a
milestone a session works normally: one task per branch and PR, and it
**merges its own PR once CI is green** (RULES 1): into `main` normally,
into `feature/track-r` for Track R. Track R milestone reviews stay on the
feature branch, pinned to an exact commit for the owner's chosen reviewer;
they do not advance `main` or `reviewed`.

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
| **M2. Perception and world model** | `Observation`, world model, knowledge tables, decision trace, parameter table | Perception agrees with the engine on recorded runs (declared exceptions only; SPEC.md §7); observations serialise and replay into the world model without the game; every engine read is listed (D3 audit, SPEC.md §5) | **done**: reviewed in #99 (Kimi K3.1 Pro, four passes; findings fixed in #100–#104), tag `m2` (2026-11-10) |
| **M3. A Borg that plays** | Executor and prompts, paths, exploration, melee, pickup, stairs, rest, town restock; the first strategy without the full tactical layer; death dossiers | **G0**: no stall, crash or harness failure on the smoke suites; deterministic replay; every natural death recorded with its decision trace | in review (#135) |
| **M4. Danger and survival** | The threat model and its calibration, reserves, escapes, the tactical layer, survival invariants | Calibration report on recorded runs; fixtures from the death cases (below) pass; early-suite deaths set beside the zoo's existing C Borg results (descriptive; no control run requested) | in progress: T4.1–T4.7 done ([calibration and gate](docs/M4-CALIBRATION.md), #124); T4.8 (#157) and the deferred M3 review findings #137–#139 done; the milestone's tasks are complete and the M4 review is next |
| **M5. Readiness and progression** (paused: the repivot) | Readiness policy per class, equipment valuation, item use, spells for casters, ranged combat | **G1** on `classes-1800` against the zoo's existing control results (owner starts the zoo run of our Borg only; prerequisites in [GOALS.md](docs/GOALS.md#observation-and-scoring-rule)) | in progress (M4 review findings #163–#165, the source refresh T5.1 #125, T5.2 #126 and the starvation residue #172 done; #127 in progress) |
| **M6. The family** | Class profiles, the first class-specific strategy where M5's data shows the need, a tuning loop (screening and racing) | **G2**, confirmed on fresh seeds | not started |
| **M7+** | Deeper suites, mid game, death-driven iterations | **G3**, then G4 | not started |

## The repivot, and tracks P and R (owner, 2026-11-14)

The POC showed that a Borg built here from first principles does not beat
borg-rs. The repository now hosts **borg-rs's decision core**
(`borg-core/`, vendored at borg-rs v12; docs/SPEC.md §2 amendment) with
**errands** around it (docs/ERRANDS.md). The engine is upstream
`42cd156ac`, the commit borg-rs plays (#216). Our own core stays the
default and is frozen: **M5–M7 are paused**, their open issues kept.

Work now runs in two tracks, side by side, each with its own milestones,
labels and review PRs (AGENTS.md *Choosing the work*). A session is
assigned to one track by the owner.

### Track P: calibration of borg-rs's core (epic #218)

Integration branch: **`track-p`**. Task PR base: `track-p`. Milestone
review/promotion: **`track-p` → `main`**, owner-reviewed before merge.
P1–P3 implementation was already merged before this rule; a combined
retrospective review covers that historical work. It does not certify
prior independent review, and main is not rewritten. Pending/new P tasks
and fixes use `track-p`; other-track PRs are not retargeted.

Named parameters in place of borg-rs's hard-coded thresholds (about 400 in
fight, flee and recovery, against 44 `borg.txt` settings), and a decision
trace. **At the defaults play is byte-identical** (full 45/45 against borg-rs
v12).

| Milestone | Label | What it delivers | Exit (all must hold) | State |
| --- | --- | --- | --- | --- |
| **P1. Fight-or-flee parameters and the trace** | `P1` | the parameter table (#219); caution (#219), danger and escape (#220), recovery (#221) as parameters; the decision trace (#222) | every threshold of those four files a parameter or listed as game data; full 45/45 at the defaults; a changed parameter changes play and the trace shows it; the trace in run outputs and death dossiers | implementation on main; owner accepted retrospective review; #260 promotion delegated (supersedes #251) |
| **P2. Attack, defence and items** | `P2` | the thresholds of `borg_fight_attack.rs`, `borg_fight_defend.rs`, `borg_item_use.rs`, `borg_power.rs` as parameters | as P1 | implementation on main (#247, #249, #250, #252); owner accepted retrospective review; #260 promotion delegated |
| **P3. Sweeps** | `P3` | the parameters as zoo profile settings (docs/ZOO-INTERFACE.md); a sweep prompt for the owner | the zoo adapter sets a parameter through a profile, checked by the zoo contract test | implementation on main (#253, #254); owner accepted retrospective review; #260 promotion delegated |

Pre-rule dependency exception: P1–P3 were already on `main` at `8351f0a3`.
For those milestones, owner acceptance of the retrospective review #259
satisfies the P review dependency; a second runtime promotion is unnecessary.
A reviewer recommendation alone is not owner acceptance. New P work still
requires the reviewed `track-p` → `main` promotion. This clarification does
not undo other-track work already authorized before the gate.

### Track R: the redesign epic (epic #217)

**Integration branch: `feature/track-r`** (owner, 2026-11-15). Every R task
branch starts from it and every R task PR targets it. The unfinished
redesign stays there across milestones. CI parity and milestone completion
are validation gates, not permission to merge R into `main`. A ready
redesign is promoted only with the owner's explicit approval. Main can be
merged into this branch to inherit Track P's parameters and fixes.

R milestone reviews use a separate feature-only review ledger:
`review/track-r-baseline`, initially at `8351f0a3` (the integration branch's
main baseline). Open each review PR from an immutable `review/Rn-SHA`
snapshot of `feature/track-r` into that ledger. Pin the review prompt to
that exact snapshot. The owner chooses the reviewer and merges review
PRs; a ledger merge does not promote code to `main` or global `reviewed`.
Stop at the milestone checkpoint under RULES 6.


borg-rs's decisions re-expressed subsystem by subsystem in our domain, in a
new crate `borg-next/`. Each port runs beside the original every think
(**shadow check**) and switches over only when the full suite's keys stay
byte-identical. Ported code reads borg-rs's memory through a typed view,
not our perception, until R6. A subsystem is ported after track P has
named its thresholds and the relevant P milestone has been reviewed and
promoted to `main`.

| Milestone | Label | What it delivers | Exit (all must hold) | State |
| --- | --- | --- | --- | --- |
| **R1. The frame** | `R1` | `borg-next` and the typed view (#223); the shadow-check harness (#224); the carry procedure tried on one borg-rs change (#225); the separate moving-reference refresh (#262) | `BORG_SHADOW=off` 45/45; a trivial port with zero mismatches in CI; the carry cost written down | **done**: accepted and merged in #266; feature-only tag `r1` |
| **R2. Fight or flee** | `R2` | danger, caution, escape, recovery ported (#209, 6,500 lines) | `check`: zero mismatches on the full suite; `switch`: 45/45; the measured report (lines, time, mismatches, carry cost); R3–R5 re-estimated from it | **done**: accepted #302/#329; feature-only tags `r2`/`r2.1`; follow-ups retained in R3 preflight |
| **R3. Attack and defence** | `R3` | `borg_fight_attack.rs`, `borg_fight_defend.rs` (8,200 lines) | as R2 | **done**: owner accepted #427 (2026-11-16); feature-only review ledger `17992dfc`; exact accepted snapshot `02c273db`; #418/#419 remain declared follow-ups |
| **R4. Items** | `R4` | power, wearing, item use, activations, shopping (9,000 lines) | as R2 | **in progress**: owner opened R4 after accepting R3; reference refresh #429; ownership audit #431; ready power/use #434/#435; wear/store #436/#437 gated on accepted P handoff #433; final review #438 |
| **R5. Paths and the think** | `R5` | flows, and `borg_think_dungeon`'s priority list into modes and goals (7,000 lines) | as R2, and the errands run on the modes frame | not started |
| **R6. Our perception** (optional) | `R6` | engine reads in place of borg-rs's screen reading | breaks parity on purpose; judged by the zoo | not started |

R2 reforecast (2026-11-15; [measured report](docs/R2-VALIDATION.md)):
6,806 physical policy lines plus 36 dispatch and 702 core-adapter lines.
The 65m38s initial issue-to-implementation envelope includes validation and
bookkeeping; it is not exclusive coding time or a lines-per-week rate.
Provisional effort relative to this R2 migration: **R3 1.5–2×**, **R4 2–3×**,
**R5 1.5–2.5×**, reflecting larger owned facts, inventory/store effects,
and modes/path/cache design. These supersede the initial calendar guesses;
the owner sets dates and the R3 task breakdown after R2 review. Track P's
schedule remains its own decision.

GitHub milestones are not used for P and R (labels are); the owner may
create them, and a session then files each issue in its milestone as well.

## Secondary track Q: the query surface

The secondary goal Q of [GOALS.md](docs/GOALS.md#secondary-goal-q-a-query-surface-for-language-models)
(D12), designed in [docs/QUERY-SURFACE.md](docs/QUERY-SURFACE.md). Its
milestones run beside the primary ones, only when the owner opens them
(RULES 2), with the same method: task issues, a review PR at the end, the
owner merges it. A Q milestone never blocks a primary one.

| Milestone | What it delivers | Exit (all must hold) | State |
| --- | --- | --- | --- |
| **Q0. Pilot** | No code: a session diagnoses deaths from the dossiers as they are (`trace-tail.jsonl`, `params.json`, `record.json`), on the reference deaths with known explanations (`reference/deaths/` bbc and rs, and the cases written up in `docs/FIXTURES.md` and REDESIGN-LESSONS) and on our own | Hit rate on the known deaths reported, by deciding **turn**, cause and alternative (the reference dossiers hold no trace or parameter table); triage table of our deaths: deciding think where the trace reaches it, the turn otherwise (`trace-tail.jsonl` keeps the last 200 thinks, `tools/export-death.py`; most of our dossiers are at that cap, and the `-c-` ones have no trace); draft fixtures for the clearest, not committed as tests; the tools it missed, which adjust Q2–Q3, a longer trace among them | not started; #153 (TQ0), runs beside M4 when the owner opens it |
| **Q1. The facade** | `borg_reborn_rs::query`: `State` rebuilt from a run's logs (observations, knowledge, params, trace), the strategy replayed with the run's seed and profile; read-only reports with units; `query::VERSION`; `harness/run.sh --record-state` | Facade tests on every decision fixture and on a recorded smoke run; replay reproduces every trace choice of the smoke references, as a CI step; play unchanged: the smoke, smoke-exact and full references reproduce key for key and trace for trace. The build's source hash (`harness/identity.sh`) does change, since it covers `borg-reborn-rs/` and `Cargo.lock`; with play unchanged that is not a new measurement baseline (RULES 22) | not started (may start once M4 is merged); tasks #149 (TQ1.1), #150 (TQ1.2), #151 (TQ1.3) |
| **Q2. The server** | `borg-mcp` workspace crate over stdio with the diagnosis tools: `load_run`, `seek`, `view`, `why`, `threats`, `danger_at`, `reserves`, `params`, `knowledge`; trace and params as resources; schemas generated and checked in | An MCP client loads a death dossier and gets cited answers (`query_version`, run identity, think, turn); CI runs the crate's tests; sealed seeds refused | not started |
| **Q3. Counterfactuals** | `responses`, `decide`, `what_if` (closed `Hypothesis` set); `branch`: rerun the run's seed and profile, take an alternative action at a think, let the Borg continue for N game turns, report what followed (a build feature the zoo never gets); claims and draft fixtures (the docs/FIXTURES.md format, not committed as tests: an alternative may not be constructible as a fixture) as outputs; a `branch` build marked in `BUILD_ID` (`features=branch`, as `test_features` is today), so its results stay with their own build (RULES 22); snapshots for fast seeking | `what_if` reproduces each death fixture's alternative; `branch` with no alternative reproduces the original `keys.log`; facade purity test (world digest unchanged after every tool at every think); a new zoo-contract assertion refuses a build whose `BUILD_ID` carries features (the check does not exist yet) | not started |
| **Q4. Diagnosis** | The workflows of docs/QUERY-SURFACE.md: evaluation on deaths with known explanations, triage of a zoo batch into cause frequencies, review of a strategy change at its divergence points | The evaluation meets the hit-rate threshold (**open: the owner fixes the threshold and the cases counted before Q4 starts**, after Q0's numbers); one zoo batch triaged, its draft fixtures and issues reviewed and filed | not started |
| **Q5. Play mode (optional)** | Live hosting; `observe`, `options`, `act`, `autopilot` with hand-off conditions; a run record per model game (model, effort, tool calls, keys); the arm's name and its death dossiers | A model plays the smoke seeds end to end, fully and in the hybrid; every game replays key for key from its record; its deaths have dossiers under its own arm; fair-information and exclusion checks pass | not started; the owner decides after Q4 whether it is wanted |

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
- Neo Angband, through Mandos's adapters.
