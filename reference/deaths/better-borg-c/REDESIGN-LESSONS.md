# Lessons from native C Borg deaths, for a ground-up redesign

Audience: **borg-reborn-rs**, the owner's planned ground-up Borg redesign, and anyone
designing a new Angband automaton. This document distils the reviewed deaths in this
repository into design requirements. It does not ask for C patches. Every lesson links back
to its evidence. Read the linked dossier before relying on a detail.

Snapshot: 2026-11-09. It covers the reviewed cases **D0001–D0014**, which are 11 distinct
starting conditions; D0003–D0005 are reference replays of D0002. It includes four verified
owner manual takeovers. Later dossiers (D0015+) are not yet reviewed. Update this file when
they are.

## How to read the evidence

| Level | Meaning |
| --- | --- |
| **verified** | A complete rerun of the recorded native game matched every input, action, clock/file channel, RNG/player state, native save and frame (`replay-borg.py --verify-only`). |
| **observed** | Read directly from the original native trace, frames or message log of that life. |
| **source-checked** | Read in the C source, pristine upstream `angband/angband` `e473720` unless noted, and consistent with the trace. The runtime branch has not been instrumented. |
| **manual counterfactual** | The owner played from a verified original prefix (save/load, no Borg memory restore). One line of play, excluded from survival counts. It shows a feasible alternative, not a policy benefit. |

Deaths here are selected cases, not a mortality sample. Do not compute survival rates from
them. The C Borg's danger estimate reads full monster race data (`r_info`), not the player's
learned lore. A redesign must decide explicitly whether it may use that spoiler knowledge.

## Case index

| Case | Character | Where | Killed by | Manual takeover | Investigation |
| --- | --- | --- | --- | --- | --- |
| [D0001](D0001-stegocentipede/report.md) | Human Warrior CL9 | DL12 (max 12) | stegocentipede | — | [#22](https://github.com/diegoami/better-borg-c/issues/22) |
| [D0002](D0002-gallant/report.md) (+D0003–5 replays) | Human Warrior CL3 | DL2 | gallant | — | [#34](https://github.com/diegoami/better-borg-c/issues/34) |
| [D0006](D0006-collected/report.md) | Human Warrior CL9 | DL14 | killer brown beetle | — | [#177](https://github.com/diegoami/better-borg-c/issues/177) |
| [D0007](D0007-collected/report.md) | Gnome Mage CL4 | DL1 | soldier | — | [#193](https://github.com/diegoami/better-borg-c/issues/193) |
| [D0008](D0008-collected/report.md) | Human Priest CL1 | DL1 | grey mold | — | [#212](https://github.com/diegoami/better-borg-c/issues/212) |
| [D0009](D0009-collected/report.md) | Half-Troll Necromancer CL10 | DL5 (max 9) | starvation | — | [#235](https://github.com/diegoami/better-borg-c/issues/235) |
| [D0010](D0010-collected/report.md) | High-Elf Ranger CL1 | DL1 | kobold | survived (bow) | [#239](https://github.com/diegoami/better-borg-c/issues/239) |
| [D0011](D0011-collected/report.md) | Half-Elf Warrior CL11 | DL18 | griffon | — | [#322](https://github.com/diegoami/better-borg-c/issues/322) |
| [D0012](D0012-collected/report.md) | Gnome Paladin CL2 | DL1 | poison (worm mass) | survived (1-hit mold) | [#325](https://github.com/diegoami/better-borg-c/issues/325) |
| [D0013](D0013-collected/report.md) | Dunadan Druid CL1 | DL1 | small kobold | survived (fled ant) | [#332](https://github.com/diegoami/better-borg-c/issues/332) |
| [D0014](D0014-collected/report.md) | High-Elf Rogue CL11 | DL20 | blood falcon | survived (Speed, CLW, detection) | [#341](https://github.com/diegoami/better-borg-c/issues/341) |

What the owner keeps saying, across reviews: the Borg had the tools and the knowledge, but
its decision layers did not use them, or worked against each other.

---

## L1. One threat model, and it should drive flight

**Failure.** Whole-level flight is triggered by hard-coded monster names and breeder counts,
which contradict the Borg's own danger estimate.
- **D0012:** two white worm masses at CL2 meant "Scary guy on level". The worms have speed −10,
  10 HP and a 1d2 bite.
- **D0013:** a single small kobold at CL1 triggered flight. Its logged danger was **5**.
  Meanwhile the giant white ant that actually killed the druid rated **18–38**, and the Borg
  aimed both of its final attacks at the ant instead of finishing the kobold.
- **D0002–D0005:** a gallant/cutpurse threat rose to danger 188–451, inflated by the theft
  weighting (#158).

**Evidence.**
- Observed: the trace notes and logged danger values in D0012 and D0013.
- Source-checked: `borg-flow-kill.c` scary scan (Grip/Fang/small kobold at CL ≤ 5; worm masses
  and mice when `breeders >= CLEVEL`). By contrast `borg-think-dungeon.c:borg_check_breeders`
  only starts leaving at `MIN(CL+2, 5)`.
- Manual counterfactual: D0012, where the owner killed the mold and the worms; D0013, where the
  kobold fled after one hit and the owner fled the ant; and D0014, where the owner tested the
  earth hounds and left.

**Owner's principle.** "One big part of surviving is knowing which monsters are too tough for
you at a level." The danger model already holds this knowledge. A second, name-based layer
then overrides it.

**Requirement.**
- Use a single per-monster and per-group threat estimate. It should account for HP, blows,
  speed, ranged and spell effects, status effects such as paralysis, blindness and confusion,
  and breeding.
- Derive the decision to flee or leave the level from that estimate, with hysteresis.
- Name special cases only as data within the model, never as a parallel trigger.
- Disengage from the monster that rates dangerous, not from the whole level by default.
- Target selection should prefer finishing weak, damaging enemies when that reduces incoming
  damage.

Issues: [#327](https://github.com/diegoami/better-borg-c/issues/327),
[#158](https://github.com/diegoami/better-borg-c/issues/158).

## L2. Decision layers must agree, and plans must persist

**Failure.** The route planner and the step-by-step caution use incompatible danger limits.
The result is back-and-forth movement that bleeds HP.
- **D0012** (observed and source-checked, matched to every logged choice). While scared at
  CL ≤ 5, the flee-to-stairs flow accepts squares with danger up to **3 × HP**
  (`borg_flow_spread` fear). Caution retreats from any square above **HP / 3**. The planner
  steps the character next to a 2-HP grey mold and caution steps it back, five times, taking
  HP from 21 to 3. Because the planner's limit scales with HP, the target stair flips:
  east at HP 17/14/9, west at 5, east at 6, west at 3.
- **D0008** (verified). Same mold shape in a different layer. An emergency flow marked a square
  "known" without evaluating its danger, and normal exploration reused it. This was fixed as
  source404 / FLOWCACHE-404. D0012 then showed the loop can still happen through the flee flow.
- **D0010** (verified). Repeated desperate stair flow pre-empted a usable bow attack.

**Requirement.**
- Use one shared danger evaluation and one consistent risk tolerance per decision, across
  planning, movement, retreat and combat.
- Once fleeing toward a chosen exit, commit to it unless the situation changes materially
  (hysteresis). Don't re-plan from scratch after every retreat step.
- Make cached map or danger knowledge carry its validity, and invalidate it on new
  observations. Source404 is the cautionary example.

Issues: [#324](https://github.com/diegoami/better-borg-c/issues/324),
[#212](https://github.com/diegoami/better-borg-c/issues/212),
[#239](https://github.com/diegoami/better-borg-c/issues/239).

## L3. Compare fighting, retreating and escaping in the same decision

**Failure.** The retreat logic runs before the attack logic, so a cheap kill is never
considered.
- **D0008** and **D0012**: a **2-HP, AC-1 grey mold** hit the character over and over without
  being attacked once. In the D0012 manual counterfactual, one blow destroyed it.
- **D0010**: an available archery kill lost out to stair flow. In the manual counterfactual,
  six arrow hits and the character survived.

**Requirement.** Every turn in contact, score all of these options together, with the same
threat model and expected outcomes:
- attack (with expected kill probability);
- step back or away;
- walk toward an exit;
- use an escape item;
- heal or buff.

Stationary monsters (molds, floating eyes) are a special case. Stepping out of reach is always
an option, and so is killing them when they are fragile.

## L4. Use the cheapest escape that is enough, and protect the last one

**Failure.**
- **D0013** (observed). A druid stood next to a floating eye, which never moves. Caution's
  escape branch read its **only** Phase Door, where one step back would have left the eye's
  reach. Before that, it had stepped *into* the doorway next to the eye it had just seen.
- **D0014** (observed). It spent its last Phase Door scroll at full HP (danger 141) and landed
  beside an orc archer and a fire spirit, with hounds and a blood falcon nearby. In the manual
  counterfactual the owner skipped the Phase entirely and survived.
- **D0002/D0004** (verified). The first Phase Door landed in a crowd. Danger rose from 188 to 451
  and repeated "critical" phases followed.
- **D0007** (verified). Five Phase Door scrolls were consumed, with the Phase spell still ready
  at the end.

**Source-checked.** `borg_caution` evaluates escape (`borg_escape`) before retreat and
back-away. Phase landing safety is sampled, but a critical fallback spends the phase anyway.

**Requirement.**
- Order escape options by cost and sufficiency: step, walk, Phase, teleport, level exit.
- Estimate landing risk from known monsters, including unseen but remembered ones.
- Treat the last escape item as a reserve with a higher bar.
- After the last escape is gone, still consider attempting a walking retreat. Don't fall back
  to "fight to the death" (#159).

Issues: [#159](https://github.com/diegoami/better-borg-c/issues/159),
[#193](https://github.com/diegoami/better-borg-c/issues/193).

## L5. Emergency resources must be usable in emergencies

**Failure.** Characters die holding the potions that fit the situation, because the rules for
using them are tuned for low danger.

| Case | Died holding | Logged situation |
| --- | --- | --- |
| D0001 | 6 known Speed | tiger then stegocentipede |
| D0002 | 12 known CLW | HP 3, danger 507 |
| D0006 | 8 known CLW | late unknown quaffs at 17 and 10 HP |
| D0007 | 5 known CLW (and a ready Phase spell) | HP 2 |
| D0011 | 8 known CLW, 2 Speed | HP 42 → 11 against a griffon |
| D0014 | **21 known CLW, 5 Speed** | danger 399–1325 |

**Source-checked mechanisms.**
- **Speed** (`borg_defend_aux_speed`): a potion is drunk only if the hasted danger falls to
  about HP/3, so never in a real emergency (#340).
- **CLW tiers** (`borg_heal`): CLW is allowed only at 30–40% down once CL ≥ 10 (D0011, #319).
  All rungs need `danger < HP + heal`, so CLW never fires at high danger (D0002, D0014).
- **Fleeing guard:** `borg_caution` calls `borg_heal` only when not fleeing, and the
  poison-emergency "quaff unknown" fallback lives inside it (D0012, D0007, #34).
- **Last-resort unknown use** requires `MAXSP == 0`, which excludes every caster (D0012
  Paladin, D0014 Rogue).
- The healing-chance adjustments run against their own comments ([neo-angband-tracker#364](https://github.com/diegoami/neo-angband-tracker/issues/364)).

**Manual counterfactual.** In D0014 the owner drank Speed on first sight of a speed +30 blood
falcon, then used CLW freely to cure blindness, confusion and bleeding. He survived at
88/91 HP, having used 1 Speed and 5 CLW.

**Requirement.**
- Value consumables by expected survival gain in the current situation.
- Buffs such as speed and resistances should be used *early*, when they change the exchange
  rate, not only when danger is already low.
- Healing should consider status cures (blind, confused, poisoned, cut) and buying turns.
- Don't hold potions while the expected outcome is death. Conserving resources is a long-run
  cost; it must never outweigh an imminent death.
- Have no class-based exclusions from emergency fallbacks.

Issues: [#340](https://github.com/diegoami/better-borg-c/issues/340),
[#319](https://github.com/diegoami/better-borg-c/issues/319),
[#34](https://github.com/diegoami/better-borg-c/issues/34).

## L6. Control depth: Deep Descent must know where it lands

**Failure.** In 4.2, Deep Descent targets **max_depth + 5**, not current depth + 5. The Borg
uses it as a generic "teleport off level" escape with no destination check
(`borg-escape.c:borg_teleport_off_level`).

| Case | Read at | Result |
| --- | --- | --- |
| D0001 | DL2/max 7 | DL12 |
| D0006 | DL4 at full HP, then again | DL9, then DL14 |
| D0011 | **town** (CL4), then DL4 at **full HP**, then DL13 | DL7, DL13, **DL18** at CL10 |
| D0014 | DL2 at **full HP**, then DL9 at **full HP**, then DL15 | DL10, DL15, **DL20** at CL9 |

Several reads came from inflated or heuristic danger: two 3-HP street urchins in D0011, a baby
green dragon in D0014.

**Requirement.**
- Every level-changing action is evaluated by its **destination**: depth versus character
  level, preparedness and stock.
- Prefer escapes that keep or reduce depth when the character is already at or beyond its
  comfortable depth.
- A deeper jump must be a deliberate choice, not a side effect of fleeing.

Issue: [#183](https://github.com/diegoami/better-borg-c/issues/183).

## L7. Be able to get home

**Failure.** Characters that know they must restock cannot reach town for tens of thousands of
turns.
- **D0011:** about 63k turns at DL4–7 with 430 "restock phase door" and 82 "need recall" notes,
  and **563** "no up stairs found, going down" fallbacks.
- **D0014:** about 21k turns on DL20.
- **D0009:** D5/D6 cycling while starving.
- Recall was deposited at Home and never carried in D0006 and D0009.

**Source-checked.** With no known up staircase, the climb goal takes a down staircase ("go
down then back up"). With connected stairs this can cycle without progress
(`borg-think-dungeon-util.c`).

**Requirement.**
- Treat a way home (Recall or a known route) as part of the preparedness state.
- Explore for up stairs before falling back to down stairs.
- Detect lack of progress (oscillating depth, repeated goals) and escalate the strategy.

Issues: [#321](https://github.com/diegoami/better-borg-c/issues/321),
[#235](https://github.com/diegoami/better-borg-c/issues/235),
[#183](https://github.com/diegoami/better-borg-c/issues/183).

## L8. Survival stock before gear; the pack is a budget

**Failure.**
- **D0013:** starting gold went on a Ring of Protection [+10] (266) and an unknown ring (50).
  It bought **one** Phase Door and **no** CLW.
- **D0001:** Phase Door was stored at Home while spare weapons and armour were carried.
- **D0011:** died carrying 4 Rings of Searching, 9 spare torches and sandals, with no Phase
  Door, Recall or teleport.
- **D0006, D0014:** slings carried with **no ammunition**.
- **D0009:** starvation.

**Implementation defect worth copying as a test.** In D0009, a quiver-backed slot-capacity
miscount (source419) produced 764 failed pickups while food ran out.

**Requirement.**
- Use a reserve policy for escapes, cures, food, light and Recall, scaled by depth and level,
  that outranks equipment upgrades.
- Account for pack capacity, including quiver-occupied slots, exactly.
- Drop or sell items with no expected use.
- A launcher without ammunition is not a weapon.

Issues: [#41](https://github.com/diegoami/better-borg-c/issues/41),
[#181](https://github.com/diegoami/better-borg-c/issues/181),
[#235](https://github.com/diegoami/better-borg-c/issues/235).

## L9. Identify what you carry, while it is safe

**Failure.** Cures are carried unidentified until death.
- **D0011:** 9 Hazy Potions were really **Cure Serious Wounds**, carried for about 47k turns.
- **D0012:** 8 Teal Potions were really **Cure Light Wounds**.
- **D0006:** unknown potions were tried only at 17 and 10 HP (Infravision, Slowness).
- **D0001:** 4 unknown Smoky Potions.

In the C Borg, unknown potions are tried essentially only in emergencies, and selling (which
identifies) needs town visits.

**Requirement.** Identify by use, scroll or sale when safe and cheap. Treat "unknown
consumable" as an information-gathering task with value, not as inert inventory.

Issue: [#180](https://github.com/diegoami/better-borg-c/issues/180).

## L10. Value equipment by what it actually does

**Failure.** In **D0011**, on arriving at DL13, the warrior swapped a Maul (4d4) (+7,+6) for a
Dagger (1d4) (+0,+0) and fought a DL18 griffon with it. That is about 21 average damage per
turn against about 7.6.

Source-checked (`borg-power.c`):
- a +1,000,000 bonus for wielding an *unidentified* weapon applies only below DL10;
- a `blows × 45,000` bonus at CL ≤ 10 swamps the real damage terms.

**Requirement.** Score equipment by expected effect in combat: damage per turn including blows,
to-hit, slays, criticals and the monster's AC, plus defensive value. Keep identification
incentives separate from combat value, and never let them invert a combat choice.

Issue: [#320](https://github.com/diegoami/better-borg-c/issues/320).

## L11. Information is a tactical resource

**Failure and contrast.**
- **D0014:** the Borg cast Detect Monsters 234 times, but only as a timed exploration chore.
  The last cast was about 6,300 turns before death, and none during the fight. The owner used it
  to choose where to retreat and recover, and to see that the earth hound pack was too large.
- **D0014:** the logged danger was **22** while a known speed +30 blood falcon (2d10/2d10/3d8)
  was 3–4 squares away out of view. It then did 63 damage before the rogue's next action.

**Requirement.**
- Use detection before committing to a route or a fight.
- Keep remembered-but-unseen monsters, especially fast ones, in the threat estimate, with
  decaying confidence rather than dropping them.
- Model speed differences as the number of enemy actions per player action.

## L12. Engineering lessons from the C carry work

This repository carries **88** verified C patch units against pristine Borg code. Many are
not tactics but plain defects:
- bounds and index errors;
- parsers that loop or misread input;
- stale caches (source404);
- miscounted capacity (source419);
- inverted comments versus code (source364);
- loops gated on the wrong variable.

See `patches/series.json` and the per-carry measurement notes in `docs/measurement/`.

**Requirement.**
- Build the redesign with unit tests per decision component, and with native-state fixtures
  taken from these deaths (each case lists the exact turn and verified branch).
- Use deterministic replay to make regressions visible.
- Keep a clear separation between knowledge state, threat model and action selection, so
  that each can be tested in isolation.

## L13. How to evaluate a redesign against this evidence

- **Fixtures.** Each dossier records the source, build, seed and options. Owner manual branches
  (`owner-manual-branch.json` in D0010, D0012, D0013, D0014) give a verified prefix and a
  known-feasible alternative. Use them as scenario tests: start from the same state, check
  that the decision is at least as good.
- **Do not count replays.** Repeated runs of the same start, reference variants and manual
  branches are not new independent lives.
- **Survival and progress together.** A policy that survives by never descending is not
  better. Use prospective paired runs on fresh seeds that measure both. See
  [DEATH-IMPROVEMENT-WORKFLOW](../measurement/DEATH-IMPROVEMENT-WORKFLOW.md) for the protocol
  used here.
- **Spoiler knowledge.** State whether the redesign's threat model uses full race data, as the C
  Borg does, or learned lore. The owner's own advantage came partly from game knowledge a new
  character's in-game recall does not show.

## Maintenance

Owner feedback on each death is recorded verbatim on its investigation issue, and processed
per [FEEDBACK.md](FEEDBACK.md). When a new death review adds evidence, update the affected
lesson's cases and evidence level here in the same PR. If a lesson's mechanism is disproved,
say so here rather than deleting the history. This repository does not dispatch work to Rust
projects; borg-reborn-rs consumes this file on its own schedule.
