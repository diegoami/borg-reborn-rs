# Lessons trace

*Where each lesson of better-borg-c's
[REDESIGN-LESSONS.md](../reference/deaths/better-borg-c/REDESIGN-LESSONS.md)
(snapshot 2026-11-09, deaths D0001–D0014, written for this repository) is
answered in our design, and which task builds it. Update this table when the
source changes (the refresh report names it, RULES 21) or when a design
choice moves.*

| Lesson | Requirement, in short | Our answer | Built in |
| --- | --- | --- | --- |
| **L1** One threat model drives flight | One per-monster and per-group estimate (HP, blows, speed, ranged, status effects, breeding) decides flight, with hysteresis. Special cases are data, not a second trigger. Disengage from the dangerous monster, not the level. Prefer finishing weak damaging enemies | GOALS principle 4; ARCHITECTURE L4 *Threat*, L5 tactical layer | M3 placeholder (#19), M4 threat model (#24 plans M4) |
| **L2** Layers agree; plans persist | One danger evaluation and one risk tolerance across planning, movement, retreat and combat. Commit to a chosen exit; caches carry validity | GOALS principle 6; ARCHITECTURE L4 *Paths*, L5 strategic layer | #18 paths, #19 strategy |
| **L3** Fight, retreat, escape in one decision | Score attack (kill probability), step back, walk to exit, escape item, heal or buff together. Stationary monsters: step out of reach, or kill if fragile | GOALS principle 5; ARCHITECTURE L5 tactical layer | M4 tactical layer |
| **L4** Cheapest sufficient escape; protect the last | Order step, walk, Phase, teleport, level exit by cost and sufficiency. Landing risk includes remembered monsters. The last escape item has a higher bar. Walking retreat stays possible after the last escape | GOALS principle 5; ARCHITECTURE L5 tactical layer (*escape ladder*) | M4 tactical layer |
| **L5** Emergency resources usable in emergencies | Value consumables by expected survival gain now. Buffs early, when they change the exchange rate. Healing includes status cures and buying turns. Never die holding the potion that fits. No class exclusions | GOALS principle 5; ARCHITECTURE L5 tactical layer (*consumable value*) | M4 tactical layer |
| **L6** Deep Descent must know where it lands | Every level change is judged by its destination (Deep Descent goes to max_depth + 5 in 4.2). Prefer escapes that keep or reduce depth when at or beyond comfortable depth | GOALS principle 8; ARCHITECTURE L5 *readiness policy* | M5 readiness |
| **L7** Be able to get home | A way home (Recall or a known route) is part of preparedness. Explore for up stairs before going down. Detect depth oscillation and escalate | GOALS principles 6 and 7; ARCHITECTURE L5 invariants and budgets | #19 strategy, #21 town |
| **L8** Survival stock before gear; the pack is a budget | A reserve policy (escapes, cures, food, light, Recall) by depth and level outranks upgrades. Exact pack capacity, quiver included. A launcher without ammunition is not a weapon | GOALS principle 7; ARCHITECTURE L2 character model, L5 invariants | #13 world model, #21 town |
| **L9** Identify what you carry, while safe | Unknown consumables are an information task with value: identify by use, scroll or sale when safe | GOALS principle 7 | #21 town, M4 |
| **L10** Value equipment by what it does | Expected damage per turn (blows, to-hit, slays, criticals, monster AC) plus defence. Identification incentives never invert a combat choice | GOALS principle 14; ARCHITECTURE L2 character model | M5 equipment valuation |
| **L11** Information is a tactical resource | Detection before committing to a route or fight. Remembered-unseen monsters stay in the threat estimate with decaying confidence. Speed as enemy actions per player action | GOALS principle 4; ARCHITECTURE L2 monster tracks, L4 threat, L5 tactical options | #13 world model, M4 |
| **L12** Engineering lessons | Unit tests per decision component; native-state fixtures from the deaths; deterministic replay; knowledge state, threat model and action selection kept separate | GOALS principles 2, 10, 11; ARCHITECTURE layers; RULES 10 | all milestones |
| **L13** How to evaluate | Owner manual branches as scenario tests; replays are not new lives; survival and progress together; state the spoiler-knowledge choice | PLAN *How every milestone from M3 on is judged*; GOALS D3 (full race data, as the C Borg) and the objective | #24 (M4 fixtures), M5–M6 comparisons |
