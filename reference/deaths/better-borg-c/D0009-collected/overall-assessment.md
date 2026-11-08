# D0009: full-pack loot retries, late food escape and starvation

## Overall picture

**Half-Troll Necromancer, character level10, dungeon level5, deepest9**, dies of
starvation at218620,HP-5/73,SP13/13. Seed2817460426, original C50 source
1cd426af4848f36d8f027bfae5114c7efcd572f6; `borg_munchkin_start=1`.
[Owner feedback](https://github.com/diegoami/better-borg-c/issues/163#issuecomment-6717711058)
is tracked in [consumer235](https://github.com/diegoami/better-borg-c/issues/235).
[Complete original-engine replay](review-verification.json) is VERIFIED across
all recorded inputs/actions/clock/file/trace/frames/native saves. This is one
life; qualification is not another independent death. The original record,
build, input, map, seven export windows and raw movie retain their actual identity.

Starvation is the final cause, but an earlier pickup loop is consequential.
The game rejects the same Bronze Wand because the effective pack is full.
Munchkin farming repeatedly walks back to it, fails pickup and starts another
one-step stair return, spending the nutrition after its last ration. Only when
weak does the special munchkin routine yield. The normal emergency fallback
then cycles throughD5/D6 without getting back to town. This supplies a concrete
capacity/pickup bug lead, not evidence that starvation is intended munchkin
behavior or that all munchkin characters share this outcome.

The character bought food earlier; ten recorded ration-eating actions end at
191708. It stored Recall at Home3279 and has no later shop transaction after3699.
At death there is no known ordinary food or carried Recall, but **five edible
unidentified mushrooms** remain, plus many unknown potions/scrolls/staves.
These are available emergency attempts with unknown effects, not proved safe
nutrition or a guaranteed rescue. The five learned Necromancer spells include
no food creation or Recall. Its13MP does not make those capabilities available.

## Decision timeline

[18 original native anchors](review-anchors.json), [qualified visible-message
changes](review-message-events.json) and [curated Borg notes](review-borg.log)
preserve actual phases. Anchors choose the last original frame at each requested
turn. Borg notes have turn stamps only; do not infer intra-turn order against
an arbitrary same-turn frame. Engine ticks are not independent player decisions.

| Turn / phase | Known resources or native event | Behavior / consequence | Alternative / limit |
| --- | --- | --- | --- |
| 11–2449 | Repeated general-store food purchase decisions/actions | Stocked rations at birth; no evidence it started this dive with no food | Purchase-quantity/stomach chronology remains distinct from reserve policy |
| 3279 | Carried Recall is sent to Home | No Recall remains in the terminal pack | Early storage is recorded; not proof that munchkin alone caused it |
| 3699 | Last recorded shop transaction | No later food restock in this life | Returning before food exhaustion deserves comparison |
| 37507–191708 | Ten ration eating actions | Last one at191708; body is still nourished after eating | Stored food and stomach nutrition are separate resources |
| 194979 tick, frame242023 | First retained post190000 "You have no room for a Bronze Wand" | Begins the repeated observed refusal sequence | Actual capacity, not merely number of lettered stacks, matters |
| 195000 tick | D6,pack21stacks,quiver30arrows+25shots; native weight1473/1199 | Full native pack; continues farming | Declutter/reserve decisions are candidates, not tested rescues |
| 200004→200017 command | Steps onto(125,20),then off toward upstairs from the same area | Item tracking says "Walking onto and deleting ... Confuse Monster", then redetects it | Native UI calls it unidentified Bronze Wand; tracker deletion is not successful pickup/drop proof |
| 205000/210000 tick | Same21stacks/55missiles; FOOD2379/1579 | Loop continues; no town progress | Earlier food-return objective versus loot admission |
| 210401 tick | "You are getting hungry." | No rations remain; no learned food spell | Investigate stock/unknown-edible eligibility before the weak threshold |
| 214792 tick | Last Bronze Wand refusal in this sequence | 764retained visible refusals from194979–214792 | It is a repeated full-pack rejection, not764new independent objects |
| 214801 tick→214804 command | "You are getting weak from hunger!", native FOOD795 | Munchkin routine logs Town-or-Weak bailout | Late mode exit does not prove earlier restock or escape intent was adequate |
| 214854 onward | "Leaving(restock) food<3" / "Fleeing(need food)" | Food need is recognized; normal fallback takes over | Not evidence that the Borg never detected hunger |
| 216801 tick | Faint from hunger | Time and food continue to advance | Distinct from source403's zero-world-clock stall |
| 218301 tick | "You are starving!!" | HP starts collapsing while escape sequence persists | No tested late rescue claimed |
| 218567/218587/218607 | D5 downstairs→D6 upstairs→regeneratedD5; some hops share one engine turn | Goes down, immediately back up, explores a step, retreats to downstairs | A safe staircase is not a route to town; D5 upstairs may be unknown |
| 218617 command→218620 death | HP1 then-5,FOOD35 at the retained final phase,13MP | Still selecting nearby stairs; starvation kills | Unknown mushrooms are legal attempts, not known guaranteed nutrition |

## Capacity failure and original-C routing

At the loop, **21pack stacks +55quiver missiles** are not two free slots.
Native `pack_slots_used()` counts regular stacks plus
`ceil(quiver_ammo / quiver_slot_size)`: `21+ceil(55/40)=23`, the whole23slot pack.
The target needs a new stack; it is a different kind from the carried Wand of
Wonder. The native failure message corroborates the calculation. Weight is also
above carrying capacity (1473 versus1199 internal tenths of a pound); burden is a
separate cost, not the explanation inferred solely from the "no room" message.

`borg_flow_take_lunal()` instead checks the nominal final pack slot
`borg_items[PACK_SLOTS-1]`. With a shorter pack caused by quiver charges, that
slot is empty although native inventory is full. The munchkin-start positive
item-value path therefore admits the target again. The Borg's inventory cheat
copy leaves that unused tail empty, and nominal empty-slot traits share the
same accounting concern. The actual product `borg-flow-take.c` is byte-identical
to pristine e473720. [Canonical source419](https://github.com/diegoami/neo-angband-tracker/issues/419)
tracks the reusable defect before any source correction and separate consumer
carry. A minimal isolated capacity fixture and corrected/negative controls
remain next source evidence; the full life and its actual failures are qualified.

The observed order is: try nearby loot, take a step toward upstairs when that
flow cannot move at cost0, then select the same unpicked object again. It is
not proved that the item was dropped and repicked. No new danger/RNG evaluations
or hidden item identities were supplied to the Borg to obtain this result.

## Preparedness, inventory and available actions

The recorded birth option is `birth_no_selling=1`. Selling for identification
and inventory relief is therefore distinct from earning money; the old
munchkin comment about income from selling must not be assumed to describe
this run.

At death the pack contains20stacks; it held21during the capacity loop, and the
quiver then charged two more pack slots. **Pack fullness changes over time**;
the terminal inventory alone would not prove the earlier loop's capacity.
There are4Mottled Mushrooms and1Crumbly Mushroom,17flavored potions,12unidentified
scrolls, several staves,4copies of the first Necromantic Tome, spare armour, a
cursed cloak, a spare main gauche and a fueled lantern. The lantern is carried
but the light slot is empty; Necromancer darkness/light decisions need class-
specific assessment, not a universal instruction to equip it. The player has a
usable halberd, bow and23arrows, plus25ironshots for no equipped sling. This
adds shared ammunition/spare-gear policy evidence without proving each stack
was legally disposable or a causal fault.

Learned: Nether Bolt, Sense Invisible, Create Darkness, Bat Form, Read Minds.
None supplies food or Recall. Half-Troll has native REGEN; `process_world()`
doubles ordinary digestion for regeneration. That is relevant food budgeting,
not a bug or a reason to assume every race has the same horizon.

The weak-food handler tries known food/nutritive known items and available
food spells. `borg_eat_unknown()` exists, but the generic late unknown-item
fallback requires high monster danger and zero maximum mana; this caster has
maxMP13 and reported localdanger0. Earlier stair returns can also pre-empt later
fallbacks. There is no recorded unknown-mushroom eating attempt in this tail.
Trying those edibles when starvation is imminent merits a scoped test; their
unidentified effects and all unfavorable results must be retained. The proposed
food/Recall/restock/identification policies should be assessed from the earlier
states, rather than treating a last-second gamble as the whole solution.

## Brainstorming and candidate fit

| Candidate / mechanism | Evidence / fit | Uncertainty / costs | Action / next evidence |
| --- | --- | --- | --- |
| Quiver-aware pickup capacity | Native full-pack messages764times;21+2slots; source last-slot check ignores dynamic capacity | Minimal isolated fixture/correction pending; do not assume one fix rescues the whole life | Source419/consumer235: full-quiver nonstackable target versus actual-room, stack-merge, gold/ammo controls |
| Loot interrupts town/food return | One-step upstairs flow repeatedly loses to same loot; no shopping after3699; food ends191708 | Exact private goals/branch transitions not additionally observed; food policy and capacity fault interact | Consumer235/shared41: isolate no-room target rejection from an earlier food reserve/return leash |
| Unknown inventory / identification | Many flavored resources stay unknown; known ordinary food empty | Purchasing/selling-identification has gold/time costs; no blanket consume-everything policy | Shared180: native knowledge/price/stock and action eligibility, with normal no-selling birth option accounted for |
| Recall reserve | Recall stored at Home3279; no later return to retrieve it | Normal low-level rules also discount Recall; munchkin item valuation further gates rewards below configured level12. Storage alone does not prove mode attribution | Shared183/41: compare carrying/reacquiring useful escape stock against slot/weight/progress costs |
| Starvation emergency unknown edibles | Five carried unknown mushrooms; native E action possible; no known food spell | Effects may worsen survival; not all potions nourish. Existing last-resort guards do not cover danger0/caster starvation | Consumer235: qualify actual guards and ordinary legal unknown-food branches before policy changes |
| Late staircase direction/intent | At weak/faint/lowHP, descendsD5 then climbsD6; no town progress | Fresh levels regenerate; known local downstairs is not a known up route. A guaranteed safe route need not exist to compare an attempt | Shared159/235: inspect earlier walking/upstairs objective and desperate stair selection |
| Stale munchkin flee flag | Early weak bailout clears mode, not the paired flag at that exact return | **Competing evidence:** depth-change update explicitly clears fleeing_munchkin. Do not label this the cause of all later hops | Defer sole-cause claim; qualify private flags only if needed to isolate remaining behavior |
| Half-Troll food demand and burden | REGEN doubles ordinary digestion; loop overweight; excess shots/spare gear | These are real costs, not proof inventory should always be minimal; ten rations were actually carried/consumed | Shared41/181: race-specific food budget and usable-ammo/gear reserve rather than universal threshold |
| Existing source18 shopping food gate | Source-checked unrelated shopping admission lead exists | No shopping after3699 means it is not demonstrated to cause this late starvation | Keep separate; no duplicate source report or asserted causal link |
| Bad strategy, luck or harness | Mode intentionally farms stairs/loot, but native refusal cycle and hunger are real and repeat exactly | No normal-mode matched run or corrected branch tested; no population inference from this collected death | Preserve exact C50 baseline; defects first, then separate prospective survival/progress evaluation |

## Priorities and next evidence

First source419's capacity fault: native scenario and narrow correction under
the source repository's own rules, then separate verified consumer carry on235.
Keep the positive/negative pickup controls and all current bug regressions.
Assess earlier food-return/resource policy and late emergency handling separately;
changing several at once would hide attribution. Existing180/183/41/159/181
cover the shared policy questions. No source/gameplay change is implemented by
this death review, and no untested stocking threshold or unknown-item heuristic
is adopted from it.

## Outcome and limits

This is not an intended "munchkin starves" outcome: farming exposes a specific
capacity/pickup bug lead and a failed recovery chain. The recording proves
resources, failed pickup, timing and death, not that any proposed escape would
survive. Source419 and consumer235 remain open/not-carried. Owner feedback is
recorded; replay-review completion is not inferred from this comment alone, so
the owner review queue remains unchanged. All original identities stay frozen.
