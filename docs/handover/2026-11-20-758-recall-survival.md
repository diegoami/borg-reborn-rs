## 2026-11-20: flight Recall survival guard (#758)

Implementation: [PR #764](https://github.com/diegoami/borg-reborn-rs/pull/764),
`task/T758-recall-survival`, worktree `.worktrees/T758-recall`, stacked on
#763 (#760/#759). Accepted main #762/#754 was synchronized without changing
its shipped-zero switches. Accepted main `09a70906` (#757/#766) was
subsequently synchronized, preserving its code/settings and resolving
parameter-citation overlaps. Zoo #511 stays pinned to the tested `5acb8104`
specimen; this main synchronization does not silently move the measurement.
#757/#761 belong to another session and were not modified by this session. #582/#583/#754/#641 and their comments were read; existing
countdown/light/exploration policies were not redone.

`caution_recall_survival` compiles/ships 0. Its named horizon is
`caution_recall_survival_delay=34` world ticks, the engine maximum, not
34 actions at every player speed. The guard uses remembered terrain and
race movement/door abilities, discounts physical approach time and sums
exposure. A caster with a route may gain line of fire before melee, so
its full approach is not credited as safe time. Observed Haste is included;
Slow is not assumed to persist.

**Owner clarification, 2026-11-20:** include reachable sleepers that may wake
before Recall. The initial awake-only version missed 807683971: at the
read at 1390 the naga was asleep; it then woke and killed the Borg at 1534.
The final forecast evaluates an awake copy of the known record, without
mutating knowledge or assuming sleep lasts for the delay. It refuses the
original read (155.61 exposure against 15 HP); that named life now reaches
100k at CL6 with Recall safety alone. This conservative bound can decline
some reads that would have succeeded before a creature actually woke.

Caution calls `borg_recall_for_flight`. On this base the escape fallback's
“Leaving (failed to teleport)” sets flags before this same caution read;
it does not independently read Recall. Both requested routes therefore
share the gate. Ordinary trip-home `borg_recall` is unchanged. A pending
Recall is checked before forecasting, matching the existing native read
prerequisite and avoiding misleading new-refusal logs while already waiting.

**Checked:** unit cases for world time/speed, travel, walls/doors and
immobile contact; CI's Rust commands and citations (0 wrong); shipped-off
smoke/reference/replay. The seven named starts were replayed locally at the
100,000-engine-turn cap. Recall-only reaches the cap for 425904011,
1976035382 and 807683971; 2095979189, 2049259432 and 1517990203 still die
with this switch alone. After the damage-first correction, all three together reach the cap in six
of seven; 1976035382 dies to a large white snake at engine turn 17873;
Rogue 390305857 already survives in the shipped control. Its old crow death
was a different, PR757 flight-on arm, not a local rescue claim.

Canonical report, all intermediate outcomes, exact builds/stream hashes,
first differing keys and natural-death records: [Cemetery #88](https://github.com/diegoami/borg-cemetery/pull/88),
`studies/reborn-flight-fights-20261120/local-cases.md`. Final wake-aware
local Recall checks used `dab3ea69`; final category/all checks used
`5acb8104`; later main synchronization retains #754's
shipped-zero additions. Final PR head/CI and the exact reviewer checklist
are on the PR. No population improvement is claimed.

**Measurement:** [borg-zoo #511](https://github.com/diegoami/borg-zoo/issues/511)
contains the exact all-switch-capable specimen, shipped control, individual
switches and all together, class scope and expected mechanisms. The owner
alone chooses tiers/scheduling. A separate owner-authorized zoo session ran the initial pilot on `78b8f297`;
no arm led on survival. That pin lacked possible-wake handling and the
later damage-first category fix. Zoo #511 explicitly supersedes it with
`5acb8104` for future runs; the corrected specimen remains unmeasured.
This session dispatched no zoo run and started no local population arm. This machine is for
builds/smoke/named-case replays (owner reaffirmed RULES 20).

**Left:** owner-selected review and owner-scheduled measurement #511.
Higher-level/range and conservative-model tradeoffs remain unmeasured.
No reviewer was selected by this session and no PR is merged without the
owner's word. Raw local artifacts remain `/tmp/T760-fight/`; canonical work
is `/tmp/T760-cemetery` on `task/T760-flight-evidence`, PR #88. Original
scratchpad inputs remain read-only. A transfer was considered, then the
owner chose waiting here; no files moved.
