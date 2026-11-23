# Plan

*Step 3 of the initialisation. The goals are in
[docs/GOALS.md](docs/GOALS.md) and the design direction in
[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md). Where this plan and the goals
differ, the goals win. Say so in your PR and propose a fix to whichever
is wrong.*

## Milestones, and when we stop for a review

**Track P exception (owner, 2026-11-15):** task PRs target `track-p` and
are merged there after green CI and the owner says its review is done
(RULES 1a). Every milestone stops for
an owner-chosen review on a pinned `track-p` → `main` PR. Review fixes
also land in `track-p`; only owner acceptance plus green CI permits
promotion to `main`, merged by the owner or expressly delegated. Do not
start the next P milestone or carry unreviewed P work onto main through
another track. CI is a compatibility gate, not independent review. The
following `main` → `reviewed` description is the historical/default flow
for tracks without their own exception; it does not apply to new P work.

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
and carries the review prompt, pinned to an exact commit, posted on the
PR. The owner names the reviewer (RULES 6; a milestone review: the owner,
or a model of another family from the one that wrote most of the
milestone), the session may launch that reviewer, and the reviewer posts
its findings on the PR itself (RULES 6). The
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
| **M3. A Borg that plays** | Executor and prompts, paths, exploration, melee, pickup, stairs, rest, town restock; the first strategy without the full tactical layer; death dossiers | **G0**: no stall, crash or harness failure on the smoke suites; deterministic replay; every natural death recorded with its decision trace | **done**: reviewed in #135, re-reviewed after #140, tag `m3` (2026-11-10) |
| **M4. Danger and survival** | The threat model and its calibration, reserves, escapes, the tactical layer, survival invariants | Calibration report on recorded runs; fixtures from the death cases (below) pass; early-suite deaths set beside the zoo's existing C Borg results (descriptive; no control run requested) | **done**: T4.1–T4.8 ([calibration and gate](docs/M4-CALIBRATION.md), #124, #157); reviewed in #162 (GLM-5.3), merged at `ae5aa11` (2026-11-13); the `m4` tag is the owner's to create there |
| **M5. Readiness and progression** (paused: the repivot) | Readiness policy per class, equipment valuation, item use, spells for casters, ranged combat | **G1** on `classes-1800` against the zoo's existing control results (owner starts the zoo run of our Borg only; prerequisites in [GOALS.md](docs/GOALS.md#observation-and-scoring-rule)) | **paused** since the repivot (2026-11-14): M4 review findings #163–#165, T5.1 #125, T5.2 #126 and the starvation residue #172 done; #127 left open |
| **M6. The family** | Class profiles, the first class-specific strategy where M5's data shows the need, a tuning loop (screening and racing) | **G2**, confirmed on fresh seeds | not started |
| **M7+** | Deeper suites, mid game, death-driven iterations | **G3**, then G4 | not started |

## The repivot, and tracks P and R (owner, 2026-11-14)

The POC showed that a Borg built here from first principles does not beat
borg-rs. The repository now hosts **borg-rs's decision core**
(`borg-core/`, vendored at borg-rs v12, now at `8c5d000a`, `borg-core/PIN`;
docs/SPEC.md §2 amendment) with **errands** around it (docs/ERRANDS.md). The
engine is upstream `574b618c` (#724; `42cd156ac`, the commit borg-rs plays,
until 2026-11-20, #216). borg-rs's core is the one that plays: shipped in
`harness/borg.txt` since #479, and the harness default since #606. Our own
core (`BORG_CORE=reborn`) is frozen: **M5–M7 are paused**, their open issues
kept.

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
| **P7. Through the first five levels** (owner, 2026-11-20, after an external review) | `P7` | Safe progress through DL2–5 on the shipped borg-rs core. Find what keeps a Borg from descending: the first failed preparedness requirement per trip (#795), and the resolved settings of every character (#798). Change only the most frequent actionable DL3/DL5 blocker, one class at a time. Measure the shipped fight and Recall switches (borg-zoo#535), the Ranger Phase Door reserve (#797, borg-zoo#536), the Blackguard reading of the Phase Door rule (borg-zoo#537) and the low-level detection exception (#796) | Two endpoints, fixed before each comparison. **Tactical changes:** deaths by 100,000 engine turns, with mean CL and depth within a prespecified margin. **Descent changes:** alive at the cap having reached DL5 (primary), with a prespecified survival margin. Every report gives rescued/newly-killed pairs, unfinished games and per-class results. Confirmation on a seed block not yet read for any decision (`classes-1800-b` has been read several times; the zoo opens a new one) | **folded into P8, not promoted** (owner, 2026-11-22: "go with 2, fold it back into track-p. We have not really released P7 anyway"). Closed for review on 2026-11-22; the review on #939 (Codex) and the fixes that followed (#959–#963, #971, #973, #976, #1001) showed it was cut too early. The fixes are on `track-p`; #939 is closed unmerged and `promote/P7` deleted. P7 is promoted with P8. Accepted 2026-11-21, PR #799; issues #795–#798 |
| **P8. Stabilize P7** (owner, 2026-11-22) | `P8` | Make P7 safe to ship: the bugs P7 brought or exposed (issues labelled `bug` and `P8`), the P7 review fixes, the measurement tasks, and P7's exit measurements. No new capability: a new one is P9 | **Stable:** a full zoo run at the `track-p` head (900 classes-1800 starts, shipped settings, as run 45028029063) has no more deaths than `main` and no new leading killer, and no `ship-blocker` issue is open. Then one milestone review of P7 and P8 together, a zoo run of the reviewed commit, promotion to `main`, and release S1 | **in progress** |
| **P9. New capabilities after P7** (owner, 2026-11-22) | `P9` | The capabilities planned during P7 that are not fixes: devices in town (#895, #918, #919), position belief (#879), stair flight through unknown grids (#881), the breeder policy (#882), what is left of confused attacks by direction (#896: the switch itself, `attack_confused_by_direction = 1`, is already shipped by #901 and part of the P8 baseline), flask saving (#912), the focused planner (#977), rubble in ball scoring (#1017). **Until P8 is promoted, their PRs may merge into `track-p` only shipped off** (every new value at its neutral setting in `harness/borg.txt`), so play and the P8 measurement are unchanged; each is turned on in P9 after its own zoo comparison | as P7 | waits for the P8 promotion |

**P7 promotion from a fixed branch** (owner, 2026-11-22): ended the same day. P7 was folded back into `track-p` (above); `promote/P7` is deleted, and the gate is again the one above: P8, with P7, is promoted from `track-p` at its milestone.

Pre-rule dependency exception: P1–P3 were already on `main` at `8351f0a3`.
For those milestones, owner acceptance of the retrospective review #259
satisfies the P review dependency; a second runtime promotion is unnecessary.
A reviewer recommendation alone is not owner acceptance. New P work still
requires the reviewed `track-p` → `main` promotion. This clarification does
not undo other-track work already authorized before the gate.

### Track R: the redesign epic (epic #217)

borg-rs's decisions re-expressed subsystem by subsystem in our domain, in a
new crate `borg-next/`, on the integration branch `feature/track-r` (its
PLAN.md holds the track's current state; this table is updated from it).
Each port runs beside the original every think (**shadow check**) and
switches over only when the full suite's keys stay byte-identical. Ported
code reads borg-rs's memory through a typed view, not our perception, until
R6. A subsystem is ported after track P has
named its thresholds and the relevant P milestone has been reviewed and
promoted to `main`.

| Milestone | Label | What it delivers | Exit (all must hold) | State |
| --- | --- | --- | --- | --- |
| **R1. The frame** | `R1` | `borg-next` and the typed view (#223); the shadow-check harness (#224); the carry procedure tried on one borg-rs change (#225) | `BORG_SHADOW=off` 45/45; a trivial port with zero mismatches in CI; the carry cost written down | **done**: accepted and merged in #266; tag `r1` |
| **R2. Fight or flee** | `R2` | danger, caution, escape, recovery ported (#209, 6,500 lines) | `check`: zero mismatches on the full suite; `switch`: 45/45; the measured report (lines, time, mismatches, carry cost); R3–R5 re-estimated from it | **done**: accepted #302/#329; tags `r2`, `r2.1` |
| **R3. Attack and defence** | `R3` | `borg_fight_attack.rs`, `borg_fight_defend.rs` (8,200 lines) | as R2 | **done**: owner accepted #427 (2026-11-16) |
| **R4. Items** | `R4` | power, wearing, item use, activations, shopping (9,000 lines) | as R2 | **in progress**: final review #438 |
| **R4.5. Catch up with the core that plays** (owner, 2026-11-21) | `R4.5` | borg-core's changes since R's pin `2f74e1c5` (P5–P7 and work merged on main: +20,119/−2,582 lines in `borg-core/src` at main `ab775d48`, the largest share, 8,565, in the six files R2/R3 own) carried into `borg-next/` up to one owner-named main pin (#856) | as R2, at that pin with the shipped `harness/borg.txt`; the carry report gives the residual backlog and re-estimates R5 | not started; needs R4 (#438) |
| **R5. Paths and the think** | `R5` | flows, and `borg_think_dungeon`'s priority list into modes and goals (7,000 lines) | as R2, and the errands run on the modes frame | not started |
| **R6. Our perception** (optional) | `R6` | engine reads in place of borg-rs's screen reading | breaks parity on purpose; judged by the zoo | not started |

Estimates (2026-11-14, to be redone after R2): R1 1–2 weeks, R2 2–3, R3–R5
3–4 each; R1–R5 about 3 to 5 months. Track P: 1–2 weeks per milestone.

GitHub milestones are not used for P and R (labels are); the owner may
create them, and a session then files each issue in its milestone as well.

## Releases (owner, 2026-11-21)

Releases ship the Borg as it plays on `main`, bundled with Angband, for people who want to watch or play with it. They do not wait for a track to finish: each release is its own milestone, built on demand by the release workflow (#819) when the owner tags it, never on ordinary pushes. The owner, on the first one: "let us do a prerelease though, we are still very buggy, S1 is too optimistic".

| Milestone | What it delivers | Exit (all must hold) | State |
| --- | --- | --- | --- |
| **S0. A preview prerelease** (`v0.1.0-alpha.1`, marked pre-release on GitHub) | The shipped borg-rs core on `main` at the release commit, with its shipped `harness/borg.txt`, class files, scary list and plans. Upstream Angband `574b618c` unchanged, with our harness hooks and frontend. A Linux package and a source archive, with licences. A short `PLAYING.md` (install, watch, stop, the main settings; the README's setup part of #816) and release notes that say plainly it is experimental | The release workflow (#819) makes a draft **pre-release**; the package, unpacked on a clean Linux machine, runs `watch-borg.sh` and a headless game as `PLAYING.md` says; smoke on the release commit matches `smoke-borg-rs`; the notes list the known problems (the Borg rarely gets past DL2-3, which is P7's work; the open bug issues) and, if a zoo run of that commit exists, its numbers as a description (RULES 14), not as a claim; the owner publishes it | not started; issues #816 (README part), #819 |
| **S1. The first playable release** (`v0.1.0`) | As S0, at a later commit, with the full user guide (#817) | As S0, not marked pre-release; and borg-zoo's measurement of that exact commit in the notes (deaths, CL, depth, alive at DL5); the owner judges the known problems small enough for a release | not started; after S0, typically once P7 is accepted |

Later releases follow the same exit at later commits.

## Secondary track Q: the query surface

The secondary goal Q of [GOALS.md](docs/GOALS.md#secondary-goal-q-a-query-surface-for-language-models)
(D12), designed in [docs/QUERY-SURFACE.md](docs/QUERY-SURFACE.md). Its
milestones run beside the primary ones, only when the owner opens them
(RULES 2), with the same method: task issues, a review PR at the end, the
owner merges it. A Q milestone never blocks a primary one.

| Milestone | What it delivers | Exit (all must hold) | State |
| --- | --- | --- | --- |
| **Q0. Pilot** | No code: a session diagnoses deaths from the dossiers as they are (`trace-tail.jsonl`, `params.json`, `record.json`), on the reference deaths with known explanations (`reference/deaths/` bbc and rs, and the cases written up in `docs/FIXTURES.md` and REDESIGN-LESSONS) and on our own | Hit rate on the known deaths reported, by deciding **turn**, cause and alternative (the reference dossiers hold no trace or parameter table); triage table of our deaths: deciding think where the trace reaches it, the turn otherwise (`trace-tail.jsonl` keeps the last 200 thinks, `tools/export-death.py`; most of our dossiers are at that cap, and the `-c-` ones have no trace); draft fixtures for the clearest, not committed as tests; the tools it missed, which adjust Q2–Q3, a longer trace among them | not started; #153 (TQ0), when the owner opens track Q |
| **Q1. The facade** | `borg_reborn_rs::query`: `State` rebuilt from a run's logs (observations, knowledge, params, trace), the strategy replayed with the run's seed and profile; read-only reports with units; `query::VERSION`; `harness/run.sh --record-state` | Facade tests on every decision fixture and on a recorded smoke run; replay reproduces every trace choice of the smoke references, as a CI step; play unchanged: the smoke, smoke-exact and full references reproduce key for key and trace for trace. The build's source hash (`harness/identity.sh`) does change, since it covers `borg-reborn-rs/` and `Cargo.lock`; with play unchanged that is not a new measurement baseline (RULES 22) | not started; tasks #149 (TQ1.1), #150 (TQ1.2), #151 (TQ1.3) |
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
  map is [docs/archive/LESSONS-TRACE.md](docs/archive/LESSONS-TRACE.md).
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

## Owner-authorized class/family preferences (#396 / #403, 2026-11-16)

The owner authorized the native P implementation after the design discussion.
#403 is complete: #405 merged into track-p at 0edee14b. Scope:
54 neutral-default class/family utility multipliers, explicit BF mapping, final
ranking seam, trace@2 and targeted validation. See docs/ATTACK-PREFERENCES.md.
No danger correction, R implementation, population dispatch or default change.
The post-merge review of task head 3f744ddb found no correctness bugs or blockers.
Review follow-up #424 clarifies weighted attack logs and this completion status.
Owner accepted #407 and delegated its promotion; it merged to main at
4a2cb910. The earlier pending status is superseded by that disposition.
Owner-accepted pinned P review precedes main promotion. R carries the accepted
contract separately after promotion and #364/#366 coordination.

## Owner-selected P4: M13 inventory calibration (#433, 2026-11-17)

**Done: owner accepted #477 and promoted P4 to main at `435b07e7`.**
The final reviewed integration head was `ac293ef7`, synchronized to accepted
main `a8ac685a` through #524. The original scope follows as history.

The owner selected #433 after #449 merged. Main was
synchronized into track-p through #474/#475 at 9cf60aae. Scope: audit the
M13 gaps in power, wear, buy/sell, store timing and home valuation; name
policy thresholds/weights with neutral defaults and bounded overrides.
Keep mechanics, UI/quantity bookkeeping, no-selling behavior and disabled
exploit paths unchanged. The explicit shopping-plan redesign #293 remains
a separate policy task, not an excuse to replace decisions in this migration.

Exit: source classification and parameter contract documented, defaults
reproduce the accepted full native 45-case streams, a non-neutral setting
changes a real native decision, registry/profile/provenance and focused
checks pass, every new natural death retained in Cemetery. Integrate task
PR #433 only into track-p, then stop for a pinned owner-selected P4
track-p→main review. R4 consumes the exact accepted pin in its own carry
(#434); green task CI alone does not authorize promotion or an R port.


## P5: validation reproducibility and evidence receipts (2026-11-18)

**Done: owner accepted the P5 milestone review and explicitly delegated
[#626](https://github.com/diegoami/borg-reborn-rs/pull/626), promoted to main
at `324697eda0c74ac0f462cfa78053d66dc97b66dc`.** Task #374/#579 integrated
at `f357404e`, #375/#608 at `7e3ea58f`, and review fixes #629/#630 at
`9e0402d4`, after green CI and owner review disposition. The final milestone
check found no blockers; the owner authorized the promotion and then
requested continuation. Earlier pending-review statements are historical.

The owner accepted the kickoff re-review and merged source refresh #555 into
track-p at `0336d44e`, then kickoff #537 at `6aa38d2b`; issues #554 and #536
are complete. The owner requested the next Track P milestone after P4 promotion. No P5 was
previously defined; this milestone closes a bounded pair of deferred review
follow-ups rather than extending gameplay policy.

1. **Source refresh (#554/#555), then kickoff (#536/#537):** source-only
   refresh synchronizes accepted main `c9e58ca0` from track-p `ac293ef7`.
   The earlier kickoff sync `6101ca01` is historical. Refresh the borg-rs
   reference/copies from `4ed3263b` to `8c5d000a`; the vendored core already
   uses the latter source. The all-six-source report, observed heads and
   explicit deferral reasons are in [the source inventory](docs/research/P5-SOURCE-REFRESH.md).
   Preserve main’s already accepted zoo pin and runtime/engine/tool/evidence
   inputs. Merge the reviewed source PR first; the kickoff then adds only
   PLAN and its handover. Subsequent tasks use the updated track-p, with any
   further needed main synchronization through its own reviewed PR.
2. **Exact validation receipts (#374):** record copyable donor/receiving
   build/run commands, environment selections and the explicit-menu suite's
   SHA-256. Identify verified historical reconstruction separately from
   contemporaneous capture. Keep the original log hashes and strict failure
   semantics; use portable checkout roles instead of local workspace paths.
3. **Reference-receipt design (#375), after #374:** assess whether future
   repeated evidence can reference an existing verified qualified package
   while retaining each attempt's source/build/config/log provenance. Deliver
   a concrete ownership/compatibility decision; implementation is conditional
   on that contract. An unsuitable reference-only format is a valid documented
   conclusion. Preserve all existing packages and published identities.

Exit: exact commands/suite identity are independently checkable, receipt
status/coverage limits are explicit, and the reference-receipt decision
preserves provenance, integrity checks and attempt-versus-life accounting.
Source keeps tooling/contracts and short evidence links; Cemetery owns new
canonical gameplay reports. No new gameplay measurement, population run,
default change, bulk migration or evidence deletion is part of this milestone.
Importer #397, recorder #409 and plan follow-ups #425 remain separate backlog.

Historical P5 gate: task PRs targeted track-p and required green CI plus
owner review disposition (RULES 1a), then a pinned track-p→main milestone
review. That gate is complete through owner-accepted promotion #626; this
paragraph records the original milestone process.


## P6: neutral enchant/decurse calibration contract (2026-11-19)

**In track-p; milestone review/promotion #822 awaits fixes and re-review.**
#680/#773, #774/#783 and #814/#818 are integrated. Review of exact
`562979d3` found the neutral implementation sound and requested a main sync,
clearer validation wording and this status update. Follow-up #860 reconciles
accepted main `ab775d48`, including #799's P7 plan, without starting P7 work
in this session. All accepted main behavior, settings and pins are retained.

[Zoo #571](https://github.com/diegoami/borg-zoo/issues/571#issuecomment-7063705400)
passed its pilot at exact `07aa4b1e` versus `a3635ad8`: 36/36 cells complete,
18/18 paired play hashes equal, ten compiled defaults verified. Those results
and the older Zoo #520 full45 remain scoped to their exact builds; neither
proves compatibility of the #860 combination with newer main. A fresh pinned
Zoo request covers that combination. P6 owner acceptance and main promotion
remain pending. R4 ownership/carry remains separate.

1. **Source startup (#706/#708):** synchronize accepted main `324697ed`
   into the next P branch, report all six moving sources and perform the
   dedicated borg-rs reference update. The corrected
   [inventory](docs/research/P6-SOURCE-REFRESH.md) records newer donor/zoo
   heads, two changed donor documents and six changed mapped core files.
   Retain the exact accepted pin/copies deliberately; a behavior-changing
   donor carry is separate. The two #680 subject files are unchanged across
   the observed donor interval. Merge
   this owner-reviewed source PR before kickoff integration.
2. **Kickoff (#707):** record accepted P5 and this bounded P6 scope. This
   documentation-only task may stack on #708, but merges after it.
3. **Enchant/decurse contract (#680), after #706 and #707:** classify all
   literals in `borg_item_enchant.rs` and `borg_item_decurse.rs` as policy,
   game mechanics, UI/index bookkeeping or existing configuration. Name
   actual policy controls only: spell-failure allowances, ammunition stack
   thresholds and enchant targets where the audit establishes policy.
   Preserve defaults, integer widths, evaluation/branch order, resource
   preference, callers and the existing `BORG_ENCHANT_LIMIT` setting.

4. **Pre-promotion reconciliation (#774), after #680:** synchronize exact
   accepted newer main through its own reviewed task PR into track-p,
   preserve the new engine/settings/other-track work and the neutral contract,
   and revalidate against that synchronized receiving baseline. The #680
   receipts at 760fae80 do not prove latest-main compatibility.

5. **Final-candidate reconciliation (#814), after #774:** synchronize exact
   accepted main07aa4b1e after the frozen #783 comparison; preserve main's
   resolved-settings receipt/test and the P6 bounds test, recheck smoke and
   a named native witness, and route the new paired comparison through Zoo571.
   Freeze that candidate for task review, then the P6 milestone review.

Exit: a source-grounded classification and parameter contract, registered
bounded overrides/profile exposure and decision-trace coverage; focused
boundary and real decision override witnesses; default compatibility against
an exact accepted receiving baseline on the native borg-rs core. Record the
suite, source/build/config identities and which streams were compared; retain
strict failures and every new natural death in Cemetery. Run the required
workspace/core/vendor and harness compatibility checks. Any gameplay evidence
belongs in Cemetery; source keeps the contract, tests and immutable pointers.
No tuning or survival-improvement claim follows from naming parameters.

R4 continues to declare enchant/decurse as shared services until a separate
carry of the owner-accepted P6 contract. Do not implement the R port here,
close R4, change shopping policy, adopt engine/control behavior, dispatch a
population experiment, migrate published evidence or sweep unrelated backlog
#397/#409/#425 into this milestone.

After P6's tasks merge into track-p with green CI and owner review disposition,
stop and open a pinned track-p→main P6 milestone review. The owner chooses the
reviewer. Keep promotion unmerged until owner acceptance, green required CI
and an explicit merge instruction. No next milestone starts before that gate.
