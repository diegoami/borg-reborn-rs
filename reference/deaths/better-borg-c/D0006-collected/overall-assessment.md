# D0006: overall picture and brainstorming

[Owner review and investigation177](https://github.com/diegoami/better-borg-c/issues/177).
Original autoexport discussion163 is retained as the work issue and links here.
Actual C40/source2cebef1b0f39, seed1344203209, Human Warrior/default point buy;
**killer brown beetle at73100, CL9/depth14/HP-6 of106**. This is the existing
natural death, not another independent attempt. Later UI/fixes do not relabel it.

## What happened

The actor's Recall scroll went to Home at111. It subsequently used town shops,
including selling unknown potions/scrolls to identify them around48189. At last
town48600 it was CL4, carried eight pack stacks/no quiver and had1gold; its
owned Recall was still at Home at death. Inventory was not full at that visit.
The next expedition exhausted Phase Door (last read66035), acquired unknown
consumables and carried a powerful Sling of Accuracy with a small ammunition
supply alongside its plain melee Dagger.

At71135 on depth4,89/89HP and modeled danger205/avoidance89, it deliberately
read an IDENTIFIED Deep Descent. The emergency escape ladder permits this after
earlier stair/teleport options. Delayed arrival71191 was **depth9**. The nearby
acolyte's scary-horrors/fear spell did not establish the cause of that drop.
Three shots existed before the71181 shot; two remained during the subsequent
snaga melee. At71445,39/100HP, it read its last Deep Descent and arrived71511
at **depth14**,37HP. This was two five-level emergency drops, not ordinary stairs.

Rest on depth14 genuinely healed it to100HP. The original sidebar did not redraw
as quickly as actual state, creating the reported HP mismatch. It resumed looking
for upstairs, retreated from a brown mold (reported danger832 at72501), then
spent its last two shots on that mold at72511/72521. Keeping an empty good sling
and choosing a melee weapon are separate questions: they occupy different slots.
No shots remained during the final encounter.

At72885 it was CL8/89HP, logging retreat/not-prepared/upstairs goals and unable
to flow to known up stairs. Captured flags after choice are leaving1/fleeing0;
textual fleeing notes do not establish a continuously set fleeing flag. The trace repeatedly raises bravery89→178→400 and chooses dark-3
exploration; the native action is numeric8(NORTH), and next position is
(43,8), with81HP at72895. Only FOOD is timed here, so confusion does not explain
the direction. [Selected native keys](review-actions.log). The original
screen also shows an open door SOUTH at(43,10) and floor(43,11), with no actual
monster on those tiles. That is an ordinary walking alternative to test even
without a guaranteed complete exit. It does not establish a successful escape:
map knowledge, pursuit, later obstructions and the possible bottom-right exit
remain unqualified. Hidden diagnostic creatures are not Borg knowledge.

The actor reaches CL9 during the final melee, then at73073/17HP drinks unknown
Clear Potion(actual Infravision); at73083/10HP unknown Black Potion(actual
Slowness). It dies73100 while still holding eight known CLW. This supports the
owner's interest in useful knowledge and earlier preparation, but does not prove
identification, a heal, Recall or a southward step alone would rescue it.

## Decision/evidence timeline

| Turn / native phase | Observation | Question / alternative |
| --- | --- | --- |
|111 command|Recall deposited Home|When should survival reserves outrank Home value?|
|48600 tick|Last town, CL4,8pack stacks,1gold|Withdraw owned Recall; audit shop-ID/purchase priorities, reserve/gold/stock choices|
|71135 command|Known DeepDescent2,89HP,danger205,3shots; no Phase/Recall|Continue known-stair retreat versus delayed deeper escape; no random unknown scroll was read|
|71191 command|Depth9,75/89HP,2shots|Retreat/restock after first drop rather than more unprepared exposure|
|71301 command|Snaga39HP,82/89HP; shots remain|Actual native ranged/melee scores, resource conservation and escape options|
|71445 command|Read last DeepDescent,39HP|Earlier return/retention versus last-moment escape; Recall itself also has a delay|
|71511 command|Depth14,37/100HP|Two-step restock/route failure now much deeper|
|71600/72000 tick|Actual43/70HP, sidebar37|Presentation178, not failed regeneration; [logical screen](snapshots/review-71600.png)|
|72441 command|Full-health rest complete; fleeing/upstairs goal|Explore safely toward a viable exit|
|72521 command|Last shot queued against brown mold|Ammunition spending/obstacle necessity versus avoiding combat|
|72885 command|89HP, south open door; northern pursuers|[Original logical map](snapshots/review-72885.png); test known-state south branch and pursuit|
|72895 command|Moved north,81HP|Why exploration/temporary bravery overrode earlier retreat direction|
|73073/73083 decisions|Unknown Infravision then Slowness; known CLW still held|Knowledge utility and actual late heal/escape eligibility, not a guaranteed rescue|

See [anchor identity/hashes](review-anchors.json), snapshots/review-TURN.{txt,json}
and their separately qualified player-known inventory JSON, plus
[filtered original events](review-events.log). PNGs render recorded logical cells;
they are not live pixel screenshots. Original7 predeath windows remain intact.

## Brainstorming: candidates, fit and costs

| Candidate | Supporting evidence | Uncertainty / cost / interaction |
| --- | --- | --- |
|[180 Identification priorities](https://github.com/diegoami/better-borg-c/issues/180)|New unknowns after last town; last-resort Infravision/Slowness|Shop-ID already exists and was used; return timing, safe opportunities, cash/stock and knowledge value need audit; excessive shopping can stall progress|
|[181 Combat readiness](https://github.com/diegoami/better-borg-c/issues/181)|Plain Dagger, good Accuracy sling, low shots preserved through snagas then depleted at mold|Not evidence that carrying sling causes weak melee; compare actual damage/blows, compatible ammo availability and target/route necessity; pack not proven to block ammo|
|[183 Recall/deeper escape](https://github.com/diegoami/better-borg-c/issues/183)|Recall at Home; deliberate4→9→14 drops while escape stock absent|Large Recall value starts afterCL7; home/prepare call order unresolved. Earlier reserve is different from a delayed Recall read under immediate lethal pressure; forbidding the only exit may worsen survival|
|[159 Walking retreat](https://github.com/diegoami/better-borg-c/issues/159)|South open door at72885, north chosen after bravery boosts|First step is physically available in recorded geometry, but full route/pursuit and Borg map knowledge untested; owner's late-doom assessment is not proof all exits blocked|
|[34 Heal eligibility](https://github.com/diegoami/better-borg-c/issues/34)|Known CLW remains; late unknown potions consumed|Share the guard/call-order question with older cases without assuming the same defect; a heal costs an action and may still lose|
|[178 Sidebar clarity](https://github.com/diegoami/better-borg-c/issues/178)|Original pixels/cells lag actualHP while resting|Confirmed viewer ambiguity; keep original cells, label pending redraw and show current/maxHP/SP; no native healing correction|
|Ordinary danger/bad luck|Two deep arrivals, pursuit and unsafe unknown rolls|Still a competing explanation; avoid selecting a lucky branch and calling it a population improvement|

## Priorities and next evidence

Fix the confirmed presentation ambiguity first. For gameplay analysis, prioritise
Recall reserve/Home decisions and destination-aware escape, then why ordinary
southward retreat was not selected. Reconstruct actual eligible native choices,
not only source thresholds. At last town/71135 use only known resources/stairs;
at72885 compare a legal south-door attempt with the actual northward action,
including failure/adverse pursuit. Do not claim instant checkpoint continuation
or a new policy success from an observational movie.

Then audit ammo purchase/recovery/shot allocation and usable melee alternatives,
and whether proactive identification/knowledge purchases add value beyond the
existing shop-ID behavior. A broader redesign is one possible outcome, not an
implementation decision. Link existing reserve41/early-restock154/source370 and
D0001's delayed-DeepDescent investigation22. Later carry167 concerns reporting an
out-of-light stair action; these late snapshots have a lit torch/spares and do
not demonstrate that mechanism. No claim the entire life matches a later build.

Every candidate implementation remains separately scoped, with native intended
behavior/controls and prospective paired survival AND long-term-progress tests,
all failures retained and untouched confirmation. Bugs remain first. No gameplay
strategy changed, no successful rescue or causal population benefit established.
