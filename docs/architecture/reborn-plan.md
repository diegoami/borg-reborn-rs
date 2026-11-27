# Borg reborn: the plan

What "reborn" means today, how the work is split into tracks, and how the
tracks hand over. The milestones, their exit criteria and their state are
kept in [PLAN.md](../../PLAN.md); the owner's decisions in
[GOALS.md](../GOALS.md); the contract in [SPEC.md](../SPEC.md). This page
is the map between them. State as of 2026-11-21. A first playable release outside the tracks, milestone S1, is proposed in PR #820.

## Where it started, and the repivot

The repository began (2026-11-08) as a Borg designed from first principles
in Rust: our own perception, world model, assessment and strategy, in
layers ([reborn-layers.md](reborn-layers.md)). It played from M3 on, and
milestones M0–M4 were built and reviewed for it.

The proof of concept showed that this core does not beat borg-rs, the
faithful Rust port of the C Borg. On **2026-11-14 the owner repivoted**
(SPEC §2 amendment, PLAN *The repivot*):

- the repository now hosts **borg-rs's decision core**, vendored at a pin
  in `borg-core/` with our declared files and hooks around it
  ([borg-core.md](borg-core.md));
- **errands** ran over that core: goals such as "go to town", "dive to
  10", a ladder or a trip plan, given in the game or in `borg.txt`
  ([ERRANDS.md](../archive/ERRANDS.md), [GAME-PLAN-LANGUAGE.md](../archive/GAME-PLAN-LANGUAGE.md));
  they and the plans were retired on 2026-11-27 (#1339);
- our own core (`borg-reborn-rs/`, `BORG_CORE=reborn`) is **frozen**, and
  its milestones M5–M7 are paused with their issues kept. Since #479 the
  borg-rs core is the one shipped in `harness/borg.txt`, and since #606 it
  is the one a run plays when `BORG_CORE` is unset;
- the redesign continues as two tracks over the shipped core.

```
              C Borg (Angband src/borg, better-borg-c fixes)
                     │ line-by-line translation (borg-rs)
                     ▼
   borg-core/  ◄── track P: name its thresholds, trace its choices,
   (ships)          measure them in the zoo; fixes land here too
                     │ accepted P milestone = a named, measured contract
                     ▼
   borg-next/  ◄── track R: re-express each subsystem in our domain,
  (feature/track-r)  run beside the original every think (shadow check),
                     switch only on byte-identical keys
```

## Track P: calibrate the shipped core (epic #218, label `track:P`)

**Goal.** Replace borg-rs's hard-coded thresholds with **named
parameters** (default, unit, range, the line that reads it, evidence) and
add a **decision trace**, without changing play at the defaults. With
those, the zoo can measure what each setting does, and a setting can be
changed in `harness/borg.txt` instead of in code.

What it built, as the shipped core has it now:

- the registry `borg-core/src/params.rs` (more than 2,000 entries) and
  its record [CALIBRATION.md](../CALIBRATION.md);
- `borg_param_*` lines in `harness/borg.txt`, per-class values in
  `harness/borg_<class>.txt`, environment and zoo-profile overrides
  ([borg-core.md §3](borg-core.md#3-the-parameter-registry),
  [ZOO-INTERFACE.md](../ZOO-INTERFACE.md));
- the trace `trace-core.jsonl` in every run and death dossier
  ([CORE-TRACE.md](../CORE-TRACE.md));
- class attack preferences ([ATTACK-PREFERENCES.md](../ATTACK-PREFERENCES.md)),
  the M13 inventory contract ([M13-INVENTORY-PARAMETERS.md](../M13-INVENTORY-PARAMETERS.md)),
  validation receipts ([VALIDATION-RECEIPTS.md](../VALIDATION-RECEIPTS.md),
  [REFERENCE-RECEIPTS.md](../REFERENCE-RECEIPTS.md)).

| Milestone | What | State (2026-11-21) |
| --- | --- | --- |
| P1 | fight-or-flee parameters (caution, danger, escape, recovery) and the trace | on `main`; retrospective review #259 accepted |
| P2 | attack, defence, item use, power parameters | on `main`; same review |
| P3 | parameters as zoo profile settings; an owner sweep prompt ([CORE-SWEEP.md](../CORE-SWEEP.md)) | on `main`; same review |
| P4 | M13 inventory calibration (#433) | promoted to `main` (#477) |
| P5 | validation reproducibility and evidence receipts | promoted to `main` (#626) |
| P6 | enchant and decurse calibration (#707) | integrated in `track-p`; milestone review and promotion PR #822 (`track-p` → `main`) open |
| P7 | through the first five levels: what keeps a Borg from descending to DL5, changed one class at a time, with a descent endpoint of its own | added to PLAN 2026-11-21 (#799) and accepted by the owner the same day; in progress (PR #820); issues #795–#798 |

**How P works** (AGENTS.md *Track P integration and review gate*): task
PRs target the integration branch `track-p`; each milestone stops for an
owner-chosen review on a pinned `track-p` → `main` PR, and only owner
acceptance promotes it. A neutral default reproduces the previous play
byte for byte; a changed shipped value is a separate, measured decision.

Besides the P milestones, owner-directed fixes to the shipped core's play
(town shopping, lightless recovery, stair loops, and others; see
[the handover notes](../handover/README.md)) land on `main` as their own
tasks. They follow the same pattern: a named parameter where a threshold
is involved, the smoke reference re-recorded when play changes, and
evidence in borg-cemetery.

## Track R: re-express the decisions in our domain (epic #217, label `track:R`)

**Goal.** borg-rs's decision logic (about 45,000 of borg-core's lines)
re-expressed subsystem by subsystem in this repository's domain: typed
state, named parameters with units and ranges, one trace line per think,
and finally modes, goals and budgets, the layers of
[reborn-layers.md](reborn-layers.md). Strategies could then change one
layer at a time instead of inside a translated C function.

**Method.**

- A new crate `borg-next/`, on the branch
  [`feature/track-r`](https://github.com/diegoami/borg-reborn-rs/tree/feature/track-r)
  (not on `main`: the whole unfinished redesign stays there until the
  owner approves its promotion; owner, 2026-11-15).
- Each ported subsystem runs **beside** borg-rs's original every think
  (the **shadow check**) and logs every mismatch. It switches over only
  when the full suite's keys stay byte-identical.
- Ported code reads borg-rs's own memory through a **typed view**, not our
  perception, so parity can hold. Our own perception (R6) would break
  parity on purpose and is optional.
- The unported part keeps taking borg-rs's and track P's changes; for the
  ported part such a change shows up as a shadow mismatch to re-port (the
  **carry**).

| Milestone | What | State (2026-11-21) |
| --- | --- | --- |
| R1 | the frame: `borg-next`, the typed view, the shadow-check harness, the carry procedure | accepted (#266) |
| R2 | fight or flee: danger, caution, escape, recovery | accepted |
| R3 | attack and defence | accepted |
| R4 | items: power, wearing, item use and activations, shopping | in progress; final checkpoint #438 |
| R5 | paths and the think: flows, and `borg_think_dungeon`'s priority list into modes and goals | not started |
| R6 | our perception in place of borg-rs's screen reading (optional) | not started |

## Track Q: the query surface

The secondary goal Q (GOALS D12): an MCP server through which language
models query the Borg's judgement or play a hosted game as a separate arm,
designed in [QUERY-SURFACE.md](../QUERY-SURFACE.md). Its milestones Q0–Q5
run only when the owner opens them and never block P or R. None has
started.

## How the tracks hand over

- **P names, R ports.** Track R ports a subsystem only after track P has
  named its thresholds and the P milestone covering them has been
  reviewed and promoted to `main` (AGENTS.md *Choosing the work*). The
  accepted P contract (parameter names, defaults, the code they cite) is
  what R reproduces; R4, for example, carries the accepted P4 inventory
  contract (#528).
- **Both share `borg-core/`.** A session works on one track, changes only
  the files its task names, and synchronizes its integration branch
  (`track-p` or `feature/track-r`) with `main` through a CI-checked PR
  without rewriting the other track's work.
- **The zoo judges.** Neither track claims an improvement from local runs:
  [borg-zoo](https://diegoami.github.io/borg-zoo/) measures every change
  that alters play (SPEC §6, RULES 14, RULES 20), and gameplay evidence is
  written in [borg-cemetery](https://github.com/diegoami/borg-cemetery).

## What this means for a reader of the code

| If you want to … | read |
| --- | --- |
| understand why the Borg did something in a game | its run's `trace-core.jsonl`, then the function it names in `borg-core/src/` ([borg-core.md](borg-core.md)), then the C original ([original-c-borg.md](original-c-borg.md)) |
| change a threshold | its `borg_param_*` line in `harness/borg.txt` (or a class file), not the code ([borg-core.md §3](borg-core.md#3-the-parameter-registry)) |
| change a decision rule | a task in the track the owner names, on its integration branch |
| see the redesigned code | `borg-next/` on `feature/track-r` |
