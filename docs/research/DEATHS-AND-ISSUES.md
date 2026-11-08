<!--
Provenance: produced on 2026-11-08 by a delegated research run (OpenCode,
opencode-go/mimo-v2.6-flash, chosen by the owner's quota service after
minimax/MiniMax-M2.7 was unavailable) from the death dossiers and the issue
bodies of neo-angband-tracker, better-borg-c and borg-rs at the pins in
pins.json. Paths below ("src/...", "digest/...") are that run's scratch
copies. Spot-checked by hand: nat#364, #376, #404, #419, #486 and bbc#34,
#41, #249, #255 match their titles and quoted text. Classifications and the
P/S column are leads, not findings.
-->

# Deaths and issues: evidence base for a from-scratch Borg redesign

Read-only digest of the two death-dossier registries (`src/better-borg-c/docs/deaths/`,
`src/borg-rs-deaths/`) and the three issue dumps (`digest/nat-issues-full.json` 291 issues,
`digest/bbc-issues.json` 135, `digest/brs-issues.json` 11). No network access; JSON processed
with python3. Every claim carries an id/number. "unknown" means the material does not say.

**Scope notes.** 10 C dossiers + 6 Rust dossiers = 16 rows. The four `seedNNNN-*.md` files in
`src/borg-rs-deaths/` are the cohort-2026-11-07 originals of D0001–D0004 (byte-same text apart
from a `../validation` vs `../../validation` path prefix and a trailing "Legacy dossier" note),
so they are not extra deaths; `cohort-2026-11-07.md` also records five lives that survived the
100k cap (not deaths). C D0003/D0004/D0005 are qualified **references of D0002's one encounter**
(`index.json` → `causes_by_distinct_start`, README "distinct starting conditions: 7"), and Rust
D0001–D0004 are four of the nine cohort births (four deaths, five alive at cap).
Classification of root cause and P/S below is mine, not the dossier's.

---

## 1. Deaths table

| id | project | class/race | CL | DL | killer | turn | dossier's own stated cause/diagnosis (quote) | root-cause family (mine) | P/S + one line |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| D0001 | better-borg-c | Human Warrior | 9 | 12 | a stegocentipede | 174862 | "original C behavior; defect not established"; "no known healing or escape stock was carried, while four Phase Door scrolls were at Home" | escape-too-late / no-escape | **P** — open actions are stock/eligibility observations (bbc22, bbc41) and the dive/restock gates are constants (nat370, nat371), so reserve thresholds address it. |
| D0002 | better-borg-c | Human Warrior | 3 | 2 | a gallant | 20445 | "original C behavior; final healing ineligible under recorded danger, defect not established"; 12 known CLW carried | crowd/breeder/summoner exposure | **S** — the recorded chain spans theft-weighted danger, emergency-phase fallback and the heal-while-flying guard (bbc158, bbc159, bbc34); no single threshold fixes all three. |
| D0003 | better-borg-c | Human Warrior (headless ref.) | 3 | 2 | a gallant | 20445 | "D0002 reference replay on 29-carry build; defect not established" | crowd/breeder/summoner exposure | **S** — reference replay of D0002's one encounter; same three-module chain, no independent cause. |
| D0004 | better-borg-c | Human Warrior (headless ref.) | 3 | 2 | a gallant | 20445 | "D0002 reference; verified headless recording/replay; defect not established" | crowd/breeder/summoner exposure | **S** — same as D0002; this dossier only adds the pinned anchors used by bbc158/159. |
| D0005 | better-borg-c | Human Warrior (visible ref.) | 3 | 2 | a gallant | 20445 | "D0002 reference; verified visible recording/replay; defect not established" | crowd/breeder/summoner exposure | **S** — same as D0002; recording method only. |
| D0006 | better-borg-c | Human Warrior | 9 | 14 | a killer brown beetle | 73100 | "policy/resource/escape investigation; native implementation defect not established; sidebar redraw mismatch confirmed" | dived too deep / under-prepared | **S** — two emergency Deep Descent drops (4→9→14) with Recall at Home and no Phase (bbc183) plus an unchosen south door (bbc159) need an escape ladder and intent change, not a constant. |
| D0007 | better-borg-c | Gnome Mage | 4 | 1 | a soldier | 179053 | "policy/call-order/escape investigation; reusable C defect not established"; five known CLW + native-ready Phase left | crowd/breeder/summoner exposure | **S** — loot/town-flight goals keep interrupting the upstairs retreat while `!goal.fleeing` wraps healing (bbc159, bbc193, bbc34): goal and call ordering. |
| D0008 | better-borg-c | Human Priest | 1 | 1 | a grey mold | 10940 | "native emergency/normal flow cache and escape-intent investigation; original-C source404 reported" | stuck/looping/stalled | **S** — proven: exception-mode cache entry reused without danger evaluation and no persistent ascend intent (nat404, bbc212); both are state-validity/intent structures. |
| D0009 | better-borg-c | Half-Troll Necromancer | 10 | 5 | starvation | 218620 | "verified starvation/resource-return failure; original-C quiver-capacity pickup defect reported as source419" | resource depletion (light/food/potions) | **S** — nat419 is local, but the loop persists because the munchkin loot objective repeatedly beats food/town return and the unknown-edible guard needs zero mana (bbc235, nat60). |
| D0010 | better-borg-c | High-Elf Ranger | 1 | 1 | a kobold | 6309 | "verified retreat/archery opportunity investigation; no reusable gameplay defect established"; 11 arrows usable | wrong-escape-choice | **S** — desperate-stair flow (accepted cost 25 for a stair 2 grids away) is admitted before ordinary attack selection (bbc239, bbc159): call ordering. |
| D0001 | borg-rs | Human Warrior | 8 | 13 | a pseudo-dragon | 37680 | "C/Rust parity observed; defect not established"; action: "Prioritize escape-stock/restocking and the decision to descend" | dived too deep / under-prepared | **P** — the dossier names exactly two levers, escape stock and the descent decision, both of which are restock/dive gates (nat370, nat371). |
| D0002 | borg-rs | Half-Elf Druid | 1 | 1 | a cutpurse | 5590 | "C/Rust parity observed; defect not established"; action: "avoiding adjacent lethal encounters with exhausted mana and minimal starting resources" | escape-too-late / no-escape | **S** — it logged "Fleeing and leaving the level (scaryguy)" and still executed an attack at 0/12 HP: escape/recovery run after attack selection in the dungeon loop. |
| D0003 | borg-rs | Hobbit Rogue | 2 | 1 | a giant yellow centipede | 5600 | "C/Rust parity observed; defect not established"; action: "emergency unknown-scroll policy and earlier avoidance" | danger-underestimate | **S** — danger `p=14` at 1/11 HP with two adjacent threats and 0 safe grids, then a gamble on an unknown scroll; needs an admission rule *and* avoidance, i.e. two modules. |
| D0004 | borg-rs | Gnome Blackguard | 1 | 1 | a wild cat | 6231 | "C/Rust parity observed; defect not established"; action: "controlled healing-versus-finishing-attack counterfactual"; 6 CLW, none quaffed | healing/cure not used or unavailable | **P** — heal-versus-finish is a direct comparison of two already-scored actions, so a score/threshold change is the natural lever (dossier's own counterfactual). |
| D0005 | borg-rs | Elf Necromancer | 2 | 1 | Fang, Farmer Maggot's Dog | 36289 | "Fang death assessed; recovery/retreat gates and spell-confidence causality unresolved"; 1 known CLW unused, legal NW step unused | escape-too-late / no-escape | **S** — source shows attack selection returning before recovery and generic stairs flow (`borg_think_dungeon.rs` 1589 vs 1643) and back-away rejecting within 2 grids of a faster foe. |
| D0006 | borg-rs | Half-Elf Priest | 2 | 1 | a grey mold | 22841 | "Grey-mold stair/exploration loop assessed; qualified private trace shows emergency movement and danger-cache reuse" | stuck/looping/stalled | **S** — three explicit retreats to the down stair each followed by north re-entry, with `fleeing/leaving/scary` all false and cache reuse (brs5, nat404): intent + cache validity. |

Family totals: crowd exposure 5, escape/no-escape 3, dived-too-deep 2, looping 2,
resource depletion 1, wrong-escape 1, danger-underestimate 1, healing-not-used 1.
**P = 3, S = 13.** No dossier needed a family outside the given list, so none was added.
No dossier claims a proved defect: 12 of the 16 diagnosis strings read "defect not established"
(or a close variant), and the other four (C D0008/source404, C D0009/source419, Rust D0005,
Rust D0006) name a reproduced or source-supported mechanism with the fix and any rescue still
open. The P/S column is therefore a judgement about where a fix would have to live, not a
proved cause.

---

## 2. Issue taxonomy — diegoami/neo-angband-tracker (nat-issues-full.json)

291 issues: 275 open / 16 closed. 263 carry label `upstream-c`, 257 carry `finding`, and
**257 carry both** (the C Borg bug catalogue); the 29 with neither label are plan/process/
upstream-seam issues. Label `bug` appears on 120, `candidate` on 200, `evidence:c-game-reproduced`
on 111. Grouping below is my rule-based classification (title first, then body, with manual
overrides), so counts are approximate; every representative number is verbatim from the dump.

### perception/update — 5 (5 open / 0 closed)
#10 "Lantern light misses the two west-side shoulder grids (inherited C bug)"; #11 "Upstream C
proposal: west-side Borg lighting — validation, duplicate audit and PR plan"; #229 "Out-of-light check never triggers on a light that is
about to burn out"; #231 "Call Light neighbour scan: the open-door case tests the Borg's own
grid and is never taken"; #290 "Queued detection can overrun resized panel arrays after a level
change".
**Shapes:** radius/edge scans that start one step off the Borg's own grid; feature predicates
that can never be true; bounds computed once and not refreshed after a level/panel change.

### memory/map — 16 (16 / 0)
#212 "Map update compares walls after the feature was overwritten, so new walls/doors never reset
the flow"; #219 "Object detection is not cleared on a new level and carries over from the previous one"; #213 "Map
update 'Floors' branch repeats the darkness test and is never taken"; #148 "CBUG-0132: Vault flow
reuses uncleared queue/cost state and lists duplicate excavation grids"; #295 "C Borg: Rune recognition
is gated by the disarmable player-trap predicate"; #312 "C Borg: the surrounded check counts trapped
walls twice and can fire with no monsters nearby".
**Shapes:** per-panel/per-level marks that survive the event that should invalidate them;
dead branches; shared scan state reused across candidates; wrong predicate gating a feature.

### flow/pathing — 15 (15 / 0)
#404 "Native C: desperate flow caches an unchecked grid and normal exploration reuses it
(grey-mold retreat loop)"; #71 "CBUG-0055: … the 'nearest up stairs' scan keeps the farthest staircase, so a nearby
item is judged off-leash"; #232 "Stair scum re-adds the up staircase under the Borg to
track_less every time"; #233 "Stair scum: down-stair distance leaks into the 'upstair is close'
test"; #146 "CBUG-0130: Spastic reverse flow stops at its origin and never produces a move";
#345 "Optimized reverse flow can prevent outlying rune selection from an existing center".
**Shapes:** values carried between sub-scans; nearest/farthest stair chosen wrongly; flows that
cannot reach their own target; entries written under one mode and trusted under another.

### caution/escape — 16 (16 / 0)
#156 "CBUG-0140: Excessive-danger fleeing guard is enabled only when ready for Morgoth"; #152
"CBUG-0136: Tracked-stair flags remain false in caution danger checks"; #154 "CBUG-0138:
Strategic retreat monster-adjacency rejection only exits its inner scan"; #155 "CBUG-0139:
Back-away monster state leaks between candidate grids"; #34 "CBUG-0016: Stair predicate queues a
climb before phase guards finish"; #157 "CBUG-0141: Recall-in-danger mana threshold compares
current mana with itself".
**Shapes:** escape guards whose condition is never true in ordinary play (or always true);
fixed ordering of the escape ladder (climb vs phase vs heal); per-candidate state leakage;
self-comparison thresholds. This group is where the deaths land.

### healing — 10 (10 / 0)
#364 "C Borg: the healing-chance adjustments run against their comments, so danger at or above
current HP makes healing less likely"; #77 "CBUG-0061: Borg hesitates to drink healing potions after the
wrong delay"; #461 "C Borg: the "save the healing rod while escape is possible" check never applies,
because rods report a fixed 10% failure"; #159 "CBUG-0143: Second healing-rod attempt bypasses the
earlier escape-reserve guard"; #486 "C Borg: borg_check_rest() rates each monster's danger at the
monster's own grid, not at the resting grid"; #80 "CBUG-0064: borg_eat_food_any(): the 'when
hurting' test is always true".
**Shapes:** inverted or dead recovery conditions (recovery *less* likely when it matters);
reserve guards bypassed by a second path; recovery timing off a stale clock; an apparent slip
that the owner ruled deliberate (#486).

### danger — 10 (9 open / 1 closed)
#69 "CBUG-0053: borg_danger() overwrites its full_damage argument with true, so the partial-damage
paths above are only reached through direct borg_danger_one_kill"; #100 "CBUG-0084: Shoot'N'Scoot
reads an unrelated monster's danger, so the Borg phases away from a harmless neighbour"; #251
"Bolt targeting takes a pass-wall monster's danger at the wrong grid"; #221 "borg_fear_spell()
doubles ouch for heavy hits but never uses it"; #61 "CBUG-0045: constitution-drain blows get their
'below 8' extra fear from strength (BI_CSTR) instead of constitution"; #250 (closed) "Teleport Other
candidate list is filled without a bound check (255 entries)" — "a stale slot can erase an adjacent
monster's danger".
**Shapes:** danger computed at the wrong point/target and then overwritten; values computed and
never read; stale entries feeding fear. Small group, but it is the model every other group scores
against.

### fighting/attack — 40 (38 / 2 closed)
#322 "Stale same-square melee targets can stop the Borg after teleportation"; #252 "Ball and arc
blast scans skip the south and east edges of the radius"; #253 "Arc targeting's angle test uses
the path end for every grid"; #254 "Missile attack with no target reads borg_items[-1] on a real
call"; #256 "Maim Foe real call returns the last monster's score, not the best one's"; #239
"Weapon slay traits are overwritten by every later equipment slot; 'kill' traits never set".
**Shapes:** edge/last-vs-best/off-by-one omissions in candidate scans; stale or dead-slot reads
fed straight to the chooser (ranged/scoring arithmetic is where much of the catalogue
lives; items/inventory is the largest group overall).

### items/inventory/wear — 54 (54 / 0) — largest group
#419 "Native C: quiver-backed full pack repeatedly retries an unpickable munchkin loot target"; #60
"CBUG-0044: borg_first_empty_inventory_slot() reports 'no room' for an empty pack, so a starving
Borg with nothing in its pack will not buy food"; #136 "CBUG-0120: … a full-pack Borg drops a
spare amulet and keeps a worthless potion"; #87 "CBUG-0071: borg_wear_stuff(): track_worn_time is
an int16_t, so borg.time.now is truncated"; #186 "Encumbrance test starts one slot past the pack,
so the just-bought exemption never applies"; #367 "C Borg: two equipment-loop guards are inert
(BI_ISENCUMB is never set …)".
**Shapes:** the Borg's pack/slot model diverging from the native accounting (quiver slots,
weight, empty-slot traits); stale slot/scan state across passes; guards on flags that are never
set.

### stores/town — 10 (10 / 0)
#23 "Urgent temple/food shop choice is overwritten or uses a stale ware target"; #19 "C home
purchase trials can leave reduced quantities after an early continue"; #20 "Recent
purchase/sale history skips its latest entry"; #29 "CBUG-0010: Home grab skip flag leaks to later
wares"; #132 "CBUG-0116: home-sell trials drain staff charges, so the Borg takes items out of its
home instead of storing a staff"; #131 "CBUG-0115: borg_think_home_sell_useful(): the two memsets … pass
sizeof(z_info->store_inven_max * sizeof(uint8_t)…".
**Shapes:** per-ware trial state leaking into the next ware; history arrays off by one; mutation
of inventory while it is being evaluated.

### depth preparation/power — 12 (12 / 0)
#49 "CBUG-0033: borg_prepared() calls borg_restock() first and returns 'Clevel < depth' …"; #370
"Quick gain: require a Phase Door (and a cure) before the first dives"; #371 "Quick gain: make the
clevel-versus-depth dive gate a tunable ratio"; #234 "borg_leave_level() reads a preparedness
reason that later borg_prepared() calls overwrote"; #235 "'Do not dive when drained' only applies
when the Borg is already heading up"; #101 "CBUG-0085: a Borg started after Sauron's death acts on
a stale power score".
**Shapes:** first-match precedence in the preparedness ladder hiding the real reason; one static
buffer shared as return state across depths; gates that only fire in one direction; stale power.

### spells/magic — 23 (23 / 0)
#516 "C Borg: borg_perma_aux_regen() casts Rapid Regeneration during the simulation pass (no
borg_simulate guard)"; #173 "borg_defend() leaves borg_simulate set when it does nothing"; #110 "Borg casts
regeneration twice or before the spell it chose"; #296 "C Borg: Necromancer spell failure estimate omits
ambient-light penalty on glowing squares"; #176 "Teleport Other defence: highest fail-rate tier is
unreachable"; #180 "Teleport Level defence falls through to the landing simulation when the spell
fails".
**Shapes:** one global `borg_simulate` flag shared by scoring and acting, leaking real casts into
the what-if pass (#110, #120, #173, #516); failure estimates that disagree with the engine
(#296); defence tiers that can never be reached.

### messages/prompts — 28 (27 / 1 closed)
#14 "combat messages stay ignored after knockout recovery"; #366 "the dig and door-bump message
handlers never match the game's texts"; #365 "neither paralysis detector fires in ordinary play
(message spelling and a > 50 turn threshold)"; #287 "Multibyte screen messages can acquire stale
text and phantom monsters"; #196 "Message parser copies the whole message as the monster name …
(who buffer unbounded)"; #13 "cancelling the map-file prompt can crash the running game".
**Shapes:** the Borg reads game state by matching message strings, and several recognizers
silently never match; unterminated/unbounded buffers then read as text (state).

### config/modes (munchkin, lunal, scum) — 21 (21 / 0)
#193 "Lunal mode is never switched off when a monster teleports the Borg"; #314 "bailing out of
stair scumming clears only one of the two lunal flags (fleeing_lunal stays set)"; #243
"Lunal/munchkin stair scans carry the nearest-stair distance over between scans"; #141 "CBUG-0125:
Malformed integer settings can leave the configuration parser spinning"; #120 "CBUG-0104: Both
functions set borg_simulate = true … return without setting it back to false"; #145 "CBUG-0129:
Ignore-reset assumes the saved ego-ignore array has already been allocated".
**Shapes:** a mode represented as a pair of flags that can disagree (#193, #314); scan locals
carried across sub-scans (#243, cf. #233); borg.txt parser fragility (#141–#145); the global
simulation flag not restored (#120).

### clock/anti-bounce/stall — 2 (2 / 0)
#246 "Borg clock-overflow reset is unreachable: the Borg aborts on \"clock overflow\" first"; #403
"C Borg: poisoned hallucinating munchkin can stair-cycle without advancing the world clock".
**Shapes:** two guards on the same counter ordered so the graceful one never runs; a mode branch
that returns before ordinary recovery while the engine does not advance time. Related stall work
sits in the consumer, not here: bbc204 (M4 munchkin seed1045 zero-turn stall) and bbc283
(`square_in_bounds(NULL)` abort).

### harness/other — 29 (17 / 12 closed — 12 of the 16 closed issues in the dump)
#360 "Tunable Borg plan review: approve after baseline, R1 and compute corrections"; #376
"Experiment toolkit for early-game survival: early-death corpus, activation checks, replay,
branching and shadow mode"; #369 "Quick gain: check that the borg.txt formula section reproduces
the built-in calculations"; #474 "C game: wizard cure-all leaves known curses, so a later Remove
Curse dereferences a NULL curse list"; #399 "borg_init sorts with a comparator that reads past its
array (heap-buffer-overflow)"; plus process/upstream seams #2, #4–#8, #15, #16, #22, #24, #223,
#299, #372, #386.
**Shapes:** plan/process/tooling; engine-side and startup defects with no gameplay subsystem;
upstream seams tracked as watches. The closed ones are upstream integrations and catalogue
refreshes, not gameplay fixes.

The 15 groups sum to the full dump: 291 issues = 275 open + 16 closed (closed: 12 harness +
2 fighting (#255, #402) + 1 messages (#195) + 1 danger (#250)).

---

## 3. Issues whose discussion says or implies the problem is architectural

No issue body says "cannot be fixed locally"; the strongest explicit signals are #360 (the only body that literally uses *Architecture* and
*structural*, and it also says *rewrite*)
and #368–#371 (all filed "before any redesign under `docs/c-borg/TUNABLE-BORG-PLAN.md`"). The
rest are included because the body shows a defect in shared state, mode validity, intent or a
duplicated contract rather than in one predicate.

| # | title | why architectural (one line) |
| --- | --- | --- |
| 360 | Tunable Borg plan review: approve after baseline, R1 and compute corrections | The only issue that literally names *Architecture* and *structural* (and uses *rewrite*); its review finds the plan's baseline-identity model contradictory and demands "separate immutable identities" before any tuning. |
| 368 | Quick gain: screen the existing borg.txt switches on frozen seeds before any code change | Filed "before any redesign"; concedes the 35 existing switches have never been measured, i.e. the current control surface is unvalidated. |
| 369 | Quick gain: check that the borg.txt formula section reproduces the built-in calculations | Two parallel implementations of the same scoring formulas must be reconciled before anything can be tuned — no single source of truth. |
| 370 | Quick gain: require a Phase Door (and a cure) before the first dives | The same preparedness rule is mirrored in `borg-prepared.c` *and* `borg.txt` (lines 1111–1112), and `borg_prepared_aux()` never asks for a Phase Door at any depth. |
| 371 | Quick gain: make the clevel-versus-depth dive gate a tunable ratio | "The dive gate is a single constant… the plan names depth preparation as probably the biggest single lever" — the biggest lever is currently hard-coded. |
| 376 | Experiment toolkit for early-game survival: early-death corpus, activation checks, replay, branching and shadow mode | States the architectural requirement outright: shadow mode "needs decision code that cannot change state while it simulates. In the C Borg, simulation is a global flag (`borg_simulate`), and several findings show it leaking state (#120, #173). A design where decision functions only read state would enforce this." |
| 404 | Native C: desperate flow caches an unchecked grid and normal exploration reuses it | Cache entries carry no mode/validity: an entry written during emergency flow is trusted by normal exploration with no danger evaluation; reproduced in both C and Rust (C D0008, Rust D0006). |
| 419 | Native C: quiver-backed full pack repeatedly retries an unpickable munchkin loot target | Two disagreeing models of the pack (native `pack_slots_used()` = stacks + ceil(quiver/40) vs the Borg's nominal last slot), so capacity can never be right from one side. |
| 403 | C Borg: poisoned hallucinating munchkin can stair-cycle without advancing the world clock | A mode branch returns before ordinary dungeon logic *and* the engine does not advance time on stair use — the stall is an interaction, not a predicate. |
| 156 | CBUG-0140: Excessive-danger fleeing guard is enabled only when ready for Morgoth | An escape guard is effectively disabled for ordinary play, and "several later escapes apply only to a fleeing Borg", so the whole escape chain hangs off one wrongly-gated flag. |
| 152 | CBUG-0136: Tracked-stair flags remain false in caution danger checks | Caution reads a flag set that the flow module never maintains — two modules own different views of "I am on a stair". |
| 314 | C Borg: bailing out of stair scumming clears only one of the two lunal flags | A mode is encoded as a pair of flags that can disagree; the caution phase and the flow-rest check kept believing lunal while the mode flag said otherwise. |
| 193 | Lunal mode is never switched off when a monster teleports the Borg | The exit test is unreachable (it sits after every monster-spell line was already recognized), and while lunal is on the Borg "skips its usual safety checks". |
| 212 | Map update compares walls after the feature was overwritten, so new walls/doors never reset the flow | The invalidation block never runs because the "old" state is read after the copy — flow/knowledge/detection stamps are never invalidated for the event that should clear them. |
| 219 | Object detection is not cleared on a new level and carries over from the previous one | Per-panel detection marks are cleared on depth change for walls/traps/doors/evil but not objects: stale state survives the only event that should reset it. |
| 234 | borg_leave_level() reads a preparedness reason that later borg_prepared() calls overwrote | One static `borg_prepared_buffer` is shared as return state across `depth`, `depth+5` and `max_depth/2` calls — aliasing of a global by design. |
| 49 | CBUG-0033: borg_prepared() calls borg_restock() first and returns "Clevel < depth" when max clevel < depth (below 50), so in borg_prepared_aux(): "1 Lite" and… | The preparedness ladder is first-match over a fixed precedence, so one reason hides all others (and #234 shows callers then read the wrong one). |
| 173 | borg_defend() leaves borg_simulate set when it does nothing | The simulation mode is one mutable global rather than a property of the call; the Rust port keeps the behaviour faithfully (see #376). |
| 516 | C Borg: borg_perma_aux_regen() casts Rapid Regeneration during the simulation pass (no borg_simulate guard) | Scoring and acting share state: `borg_spell()` ignores the simulate flag and queues keys immediately, so evaluating an option performs it (#110, #120 are the same shape). |
| 461 | C Borg: the "save the healing rod while escape is possible" check never applies, because rods report a fixed 10% failure | A policy guard is dead because its reliability input is a constant (rods always report 100): the policy exists but its premise can never hold. |
| 364 | C Borg: the healing-chance adjustments run against their comments, so danger at or above current HP makes healing less likely | The recovery heuristic's arithmetic contradicts its own stated design, i.e. the spec and the code have drifted apart with no test that compares them. |
| 486 | C Borg: borg_check_rest() rates each monster's danger at the monster's own grid, not at the resting grid | An apparent local slip that the owner ruled deliberate ("this is not a bug fix") — the evaluation point of danger is a design decision nobody wrote down. |
| 287 | Multibyte screen messages can acquire stale text and phantom monsters | The Borg derives world state from screen text; an earlier fix shape for this very issue wrote 177 bytes past the buffer — the perception channel itself is the fragile layer. |

---

## 4. better-borg-c strategy proposals/investigations and owner feedback

**Strategy proposals and investigations (all OPEN unless noted):**

| # | one-line summary |
| --- | --- |
| 41 | Owner priority: "carry/replenish usable escape/healing/survival resources ahead of spare weapons/equipment" — motivated by D0001 (3 spare weapons, 4 armour, 19/23 slots used, 4 Phase Door at Home). |
| 154 | Evaluate requiring ≥1 Phase Door (variant ≥2) and ≥1 cure before early dives, linked to nat370; notes D0004 already had 4 Phase and 12 CLW at 20293, so departure stock alone would not prevent that death. |
| 158 | Analyse danger valuation: gold theft adds 50 at CL<5 in `borg_danger_physical`; test whether lowering it changes the fight/walk/Phase choice at 20293 (danger 188). |
| 159 | Analyse early room retreat: why walking/flow/retreat lost to critical-phase fallback at 20303 (danger 451), including after scroll exhaustion at 20323. |
| 177 | D0006 owner review: identification, ammunition, town escape and two Deep Descent jumps (4→9→14) with Recall at Home. |
| 180 | Identification-driven inventory and town trading as a survival policy; "Compare local policy improvements against a redesign; neither is assumed necessary." |
| 181 | Ammunition reserves, target spending and melee readiness before deeper exposure (D0006's last two shots spent on a brown mold at depth 14). |
| 183 | Early Recall retention and destination-aware Deep Descent escape; answers the owner's "why deliberately read identified Deep Descent instead of returning to town?" |
| 193 | D0007: native-ready Phase Door and five known cures skipped during the fatal mage melee; admission/call order unobserved. |
| 212 | D0008: retreat/re-exposure loop and unchecked flow-cache reuse (carried as patch 0049, PR222; population benefit still open). |
| 235 | D0009: full-pack munchkin pickup loop, delayed restock and failed food escape (nat419, 764 recorded refusals). |
| 239 | D0010: blocked retreat pre-empting usable archery (flow cost 25 for a stair 2 grids away, 11 arrows carried). |
| 155 | The evidence gate itself: diagnosis and "a separately scoped candidate that improves survival **and** long-term progress" are two distinct processes; "A rescued replay or green unit tests alone cannot establish improvement." |
| 204 | M4 munchkin seed 1045 zero-turn stall reported by borg-zoo (Borg decisions increase while the engine does not advance). |
| 249 | Death *after* the excessive-danger flee carry (nat156): seed 29 alive at cap before patch 0053, dead to a cutpurse at turn 15149 after — a fix that opened a worse path. |
| 255 | Death *after* the back-away carry (nat155): seed 47 dies after patch 0056 while "the same patch SAVES seed 29"; net "2 alive before (11, 47), 2 alive after (11, 29): no claim in either direction." |

**Owner feedback quoted from FEEDBACK.md and issue bodies:**

- FEEDBACK.md, D0001: "prioritize usable escape/healing/survival stock over spare gear… Its
  terminal pack had 19 occupied/23 slots (four free), and Phase Door was carried and consumed
  earlier. … Slot pressure or departure with zero scrolls is not established from the terminal
  dump." (proposal bbc41, decisions bbc22)
- FEEDBACK.md, D0002: "exposure to too many enemies too early for level3/resources… A recorded
  Phase Door precedes the critical crowd; intentional walking into it is not assumed. No new
  crowd threshold or claimed rescue is implemented from one case." (bbc34, comment 6685824152)
- bbc34 comment 6709969033 (dossier `D0004-named-headless/owner-phase-feedback.md`): the owner
  "disputes BOTH the first Phase Door at 20293 and the response to landing in a crowded room at
  20294", proposing "fighting the manageable single opponent initially, or leaving the landing
  room through its door immediately and seeking upstairs, even via unexplored ground."
- bbc163 comment 6713938482 (C D0007 `overall-assessment.md`): "leave the crowded room early
  rather than continue fighting with whatever resources remain; there are too many enemies" —
  which the assessment makes "the central policy question under 159", with 193/34 explicitly
  "not substitutes for avoiding the crowded fight."
- bbc177 (C D0006): the owner "carries too many unidentified potions rather than combat-useful
  stock; prioritise identifying items, returning to town/selling to identify, and buying cheap
  potions solely to learn their identity"; "They would avoid fights, seek stairs/Recall/town
  earlier"; at 72885 the owner "believes survival may already be impossible".
- bbc163 comment 6718075726 (C D0010): owner saw Phase Door at 6255, a return toward the stairs
  and "no fighting back"; the owner's own saved alternative survived to turn 7303, CL2, HP12 with
  "six confirmed arrow hits and a melee yeek finish". bbc239 adds the owner preference: "no more
  new munchkin collections unless explicitly re-enabled."
- brs3 comments (Rust): D0005 owner feedback 6713459074 proposes the NW walking step at 36234;
  D0006 owner feedback 6714194885 reports the retreat/re-exposure loop, no fight, nine torches and
  the ready stair. Both assessments answer them without claiming a rescue.
- FEEDBACK.md process preferences: replay offsets are "**5,12,25,40,60,82,105** engine turns
  before death", with "Ten is intentionally omitted"; "A terminal picture alone does not prove how
  it entered the situation"; "A merged policy alone does not prove a survival benefit"; "Bug
  corrections always take priority."

---

## 5. Cross-cutting lessons for a from-scratch redesign

1. **Escape/healing stock must be a continuously maintained reserve, not a depth-gated restock
   check.** Phase Door is only requested from depth 6 and never by `borg_prepared_aux()`
   (nat370); D0001 died at depth 12 with 4 Phase Door at Home and 6 Speed unused (bbc22/bbc41),
   Rust D0001 at depth 13 with no Phase after a Deep Descent read, Rust D0006 sold both starting
   CSW and stored Recall at Home (brs5), C D0006 kept Recall at Home through two emergency drops
   (bbc183).
2. **Recovery must not be subordinate to attack selection or a fleeing flag.** The dungeon loop
   returns from attack selection before reaching recovery and generic stairs flow
   (`borg_think_dungeon.rs` 1589 vs 1643, cited in Rust D0005's assessment); C D0002 died with 12
   CLW because `danger < hp + clw_heal` was false (bbc34), C D0007 with 5 CLW and a native-ready
   Phase behind `!goal.fleeing` (bbc193), Rust D0004 with 6 CLW, Rust D0002 fleeing-but-attacking
   at 0/12 HP. nat364 shows the heal-chance arithmetic itself runs against its comment.
3. **Danger must separate economic/status risk from lethal risk, and be evaluated at the right
   point.** Theft adds 50 at CL<5 and group modifiers multiply it (bbc158), a candidate
   contributor to the level-3 warrior's danger 188 then 451 and its four burned Phase Door
   (D0002 anchors; the exact decomposition is "NOT yet established" per bbc158);
   nat69 overwrites `full_damage`, nat100 reads an unrelated monster's danger, and nat486 shows the
   evaluation point of rest-danger was only ruled deliberate by the owner after the fact.
4. **A mode must be one coherent object with explicit entry, exit and cache invalidation.** Lunal
   is two flags that can disagree (nat314), its exit test is unreachable (nat193), munchkin stair
   scans carry distances across sub-scans (nat243), and emergency-flow cache entries are trusted
   by normal exploration (nat404) — the exact mechanism of both grey-mold loop deaths, C D0008
   and Rust D0006.
5. **Intent must survive an emergency step.** C D0008 reached the known upstairs five times with
   `fleeing/leaving/scary/stair_less` all false and then walked east again; Rust D0006 made three
   explicit retreats before the fourth exposure; C D0007's town-flight objective was repeatedly
   interrupted by loot (bbc159); C D0009's one-step upstairs flow kept losing to the same
   unpickable wand (bbc235). nat152/nat234 show the stair/preparedness flags that should carry
   intent do not.
6. **Scoring must not mutate live state; decision functions should read only.** Shadow mode is
   impossible while `borg_simulate` is a global flag (nat376's own words), and the flag already
   leaks in nat120, nat173, nat110 and nat516 (an option evaluated during scoring gets cast).
7. **One model of the pack, and survival stock ranks above spare gear.** Native capacity counts
   quiver charges, the Borg checks a nominal empty slot, and the mismatch produced 764 pickup
   refusals and starvation (nat419, C D0009); the mirror case is nat60, where an empty pack is
   reported as "no room" so a starving Borg will not buy food. The owner's priority order is
   explicit: survival stock over spare gear (bbc41).
8. **The preparedness ladder needs one tunable source of truth with named reasons.** Reasons are
   first-match over a fixed precedence (nat49), aliased through one static buffer (nat234), mirrored
   between C and `borg.txt` (nat369, nat370), and some gates only fire in one direction (nat235);
   nat371 identifies depth preparation as "probably the biggest single lever".
9. **Walking retreat must be a real competitor in action selection, judged by cost, not by
   geometric proximity.** C D0010 accepted a flow cost of 25 for a stair 2 grids away while
   carrying 11 arrows (bbc239); C D0006 had an open south door at 72885 and chose north (bbc159);
   Rust D0005 had a proved-legal NW step and chose melee (brs4); nat153/nat154/nat155 show the
   retreat scanners themselves dropping candidates.
10. **A parameter change is not a policy result — and fixes interact.** bbc249 shows the source156
    flee carry turning a seed 29 alive at cap into a death, and bbc255 shows the source155
    back-away carry killing seed 47 while "the same patch SAVES seed 29" ("no claim in either
    direction"); bbc155
    requires separate diagnosis and prospective paired survival *and* progress evaluation, and
    nat376 asks for the corpus, activation checks and shadow mode to answer "would this fix have
    fired?" before anything is adopted.
