# Gallant encounter: overall assessment and brainstorming

This is the current shared evaluation for the qualified D0002–D0005 family,
using D0004 original C34 source1b95154f65b2 and its verified rerun. Each dossier
retains its own source identity; this is one encounter family, not four
independent deaths. [Original anchors/source analysis](owner-phase-feedback.md).
Owner suggestions are tracked on investigation34 and focused candidates158/159.

## What the trajectory shows

A level3 Human Warrior with46/49HP, four known Phase Door and twelve known
Cure Light Wounds faces an adjacent21HP cutpurse. At20293 the Borg's heuristic
danger188 and fleeing/scary-guy state lead to Phase Door. It lands20294 in a room
with several visible enemies. At first next command20303 danger is451. The
critical fallback spends more Phase Doors despite failed safety samples,
exhausting them20323. At20430 it has22HP; a gallant and one scout are adjacent,
another scout is two squares away. It eventually dies to a gallant20445 with
CLW remaining. The existing final-heal analysis finds recorded danger incompatible
with the CLW safety guard; exact runtime call order remains unresolved.

The native fatal cause is melee. The earlier contributing chain is provisional:
possibly excessive economic-threat valuation → first jump → crowd exposure →
repeated emergency phases rather than a walking attempt → no scrolls → late
melee where healing may no longer be viable. Each arrow needs its own evidence.
A bad landing, actual pursuit/blockage, group danger or intended emergency rules
are competing explanations. This does not establish one root bug.

## Decision windows and alternatives

| Turn / phase | Known observation | Alternative / unanswered question |
| --- | --- | --- |
|20293 command|HP46, Phase4, CLW12, danger188, visible adjacent cutpurse|Could fighting or walking be preferable? How much of188 is theft versus group/fear and bodily risk?|
|20294 tick →20303 command|Landing, then next decision HP46/danger451; Phase3|Attempt room exit/door/upstairs route before more phases; which enemies were asleep, what route was known, why did the chooser reject walking?|
|20313/20323 decisions|Danger401/427, critical-phase chain consumes remaining scrolls|Reassess walking at each actual decision; was exit becoming blocked or merely unattractive to the scorer?|
|After20323 →20430 late fight|No Phase Door;20430 HP22, nearby enemies|Attempted escape remains an alternative. Compare walking/door movement with staying/fighting even if neither is guaranteed safe; establish legal exits and pursuit costs.|
|Final decisions before20445|Healing stock remains; high danger and fleeing|Observe actual heal/defend eligibility and action order; distinguish a too-late heal from an earlier recovery opportunity.|

Sleeping-enemy/wake timing is the owner's observation, not verified by existing
monster metadata. The original map/actual-state diagnostics cannot alone prove
what the Borg knew or that a door route succeeds. Unknown terrain does not
justify omitting a retreat attempt from the comparison. A genuinely blocked
move should be demonstrated, not inferred from lack of a safe complete route.

## Candidate improvements and how they fit

| Candidate | Why investigate | Missing evidence / possible cost | Fit |
| --- | --- | --- | --- |
|[158: theft danger](https://github.com/diegoami/better-borg-c/issues/158)|Low-level theft adds50 and group modifiers can amplify risk;188 is not incoming HP damage|Exact components and decision sensitivity; lower weight could lose buying power or understate actual threats|Potentially avoid the first unnecessary jump; not yet proved unnecessary|
|[159: attempted room retreat](https://github.com/diegoami/better-borg-c/issues/159)|Crowd landing followed by critical phases; owner proposes door/upstairs escape|Awake state, known/legal route, chooser ordering, obstruction and pursuit; walking may also lose|Test before repeated phases AND after last scroll; does not depend on guaranteed safe route|
|[154/source370: early reserve gate](https://github.com/diegoami/better-borg-c/issues/154)|Having/restocking escape and cures may reduce shallow deaths|This actor already had both at first jump; later restock opportunity and town/progress costs unresolved|Availability policy, separate from using stock well|
|[41: stock versus spare gear](https://github.com/diegoami/better-borg-c/issues/41)|Owner survival-stock priority, motivated also by other family|Need actual inventory/gold/consumption constraints; terminal stock alone not proof|Broader reserve policy; do not conflate with this first jump|
|[34: heal/defend timing](https://github.com/diegoami/better-borg-c/issues/34)|Twelve CLW remain at death|Actual guards/call order and earlier eligibility; drinking may cost a fatal action|Recovery question downstream of exposure; no tested rescue|
|Existing rules / bad luck|Phase landing can worsen the position despite a passing heuristic check|Need ordinary legal comparisons and adverse branches; hindsight uses hidden knowledge|A possible no-bug result, not a reason to skip alternatives|

## Priorities and next evidence

First reconstruct the exact original known state and danger components at20293,
and room knowledge/sleep/action ordering at20303. These independent observations
separate the158 valuation question from159 route-selection question. For159 also
inspect decisions after20323 and near20430: no consumable does not mean no
attempted escape. Then run labelled legal fight/walk/phase branches from the
qualified original prefix, keeping unfavorable outcomes and RNG divergence.
Do not change theft weight and retreat policy together before identifying why.

Concrete discovered implementation bugs pre-empt policy work and route through
source207 when reusable. Enhancement implementation remains a separately scoped
experiment; survival/progress and untouched-confirmation gates apply. No new
strategy, successful escape or population improvement is established here.
