# D0006 overall assessment

Status: original sequence and selected private control flow assessed. A qualified
read-only trace identifies the retreat/exploration/cache interaction; a successful
alternative action and a minimal correction remain untested. Follow-up:
[Rust investigation #5](https://github.com/diegoami/borg-rs/issues/5).
This answers the [owner's feedback](https://github.com/diegoami/borg-rs/issues/3#issuecomment-6714194885).

## Overall picture

A Half-Elf Priest, character level2 with15HP/4MP, returns from dungeon level2
to level1, rests on a down staircase, opens the north door and sees a grey mold.
The Priest enters the adjacent doorway four times. After the first three
exposures it explicitly retreats to that same stair; each safe arrival is
followed by exploration back into the doorway. The fourth exposure kills it
at22841, HP-2/15 and MP3/4. It never attacks the mold and the mold remains2HP.

The immediate cause is the mold's adjacent spore melee. The strongest observed
mechanism is a mismatch between **temporary emergency movement and subsequent
exploration**, reinforced by reuse of a danger-cache entry made during the
emergency flow. At the doorway, danger16 triggers the desperate stair step
before attack selection; at the stair, current danger0 and no durable fleeing
or descent flag permit exploration again. The ordinary exploration flow then
reuses the doorway's `know=1, icky=0` entry without evaluating its danger again.
This is observed private control flow on a replay that preserves every original
action/frame/save. It is stronger than source speculation, but does not yet
prove that changing one guard or cache rule safely resolves the life.

Earlier resource decisions matter separately. It sold its two starting Cure
Serious Wounds in town, stored Recall at Home, and spent its one bought Phase
scroll well before the encounter. It retained nine spare torches and two
unidentified potions. No healing prayer was learned. This death does not show
that it refused a known carried cure, or that the pack was full. Why the
inventory valuation sold cures and prioritized fuel needs its own score trace.

### Evidence identity and qualification

Original gameplay source: `aa4ad6cfeea2cf19870e224219dfe66688175c6e`;
source SHA-256 `91874dd5bf2cd8317dadb53193b99b10ee2c06a948e2dfb9f5b2d27ef075af42`;
birth seed2647001198. Original binary SHA-256:
`84cece329fd2d9abeca1547da9ea9e5289c2d6edb1ce1f00ebc0db0a5b83b44d`.
[record.json](record.json) and its24 hashed artifacts remain unchanged.

The original recording already contains native known inventory and prayer
views. A frozen-original-binary rerun matches keys, messages, player state,
all1556 raw frames, final native save, configuration and all22 scheduled living
saves. Every native archive byte matches except the explicitly qualified
first-line death-log wall-clock date. [analysis-evidence.json](analysis-evidence.json)
retains the proof/hashes, raw paths and21 exact-frame anchor bindings;
[analysis-validation.json](analysis-validation.json) records curation checks.

A second isolated build exports that exact historical source and adds only
read-only stderr observations in five Rust files. All1360 source files shared
with the original prepared source were compared; only those five declared
files differ. [private-gates-observer.patch](private-gates-observer.patch) shows
the complete instrumentation, [build identity](private-gates-build.json) binds
source/patch/binary hashes, and [qualification](private-gates-qualification.json)
proves the same original channels, raw frames and native saves still match.
It reads existing fields and already-computed danger, without executing an
extra candidate decision or changing tactics. The42 selected trace rows are in
[private-gates.jsonl](private-gates.jsonl); existing danger and flow-threshold
values are in [private-gates-auxiliary.log](private-gates-auxiliary.log).
This is not a whole-private-Borg or per-frame whole-RNG shadow, another life,
or an alternate-action experiment. The verifier's generic extension field list
does not mean new inventory fields were added to D0006: raw frames match exactly.

### Geometry and the owner's proposed escape

Coordinates are `(x,y)`; historical Borg `Loc:y,x` notes use the reverse order.

```text
     (32,47) .   (33,47) .   (34,47) grey mold
     (32,48) #   (33,48) '   (34,48) #       doorway/exposure tile
     (32,49) #   (33,49) >   (34,49) #       down staircase/refuge
                 (33,50) #
```

The visible mold is stationary, native speed110 and2HP. Its definition has
two `SPORE:HURT:1d4` blows and inherits `NEVER_MOVE` from the mold base. These
spores are adjacent melee, not a ranged spell or cloud that reaches the stair.
The Priest is also speed110 in terminal Borg traits. At the stair, the mold is
distance2; in the doorway it is diagonally adjacent. The captured native terrain
and actual repeated traversal prove the south retreat and north re-entry are
legal. The small stair pocket has no other known exit except the doorway or
downstairs. No hidden monster positions are used to invent a route.

The owner's ready-stair observation is right, with a direction correction:
this is a **down** stair from D1 back to D2, not an upstairs route to town.
A descent is a legal alternative attempt, but its new-level danger and eventual
return/progress have not been tested. Holding the distance2 refuge or closing
the clear adjacent door are also alternatives to examine. A stationary melee
monster cannot pursue onto that refuge by itself; that fact does not prove
indefinite safety from all future dungeon events or successful recovery.

### Actual resources, learning and mace viability

At the encounter the pack has the Novice's Handbook, one Apple, seven Rations,
a Cloudy Potion, a Muddy Potion, eight full Wooden Torches and one partly burned
torch. Another torch is equipped. There are seven pack entries, no quiver or
launcher, carrying weight463 with616 capacity remaining in native units. The
potions are **unidentified during life**. Postmortem reveal names Blindness and
Speed; using those names to recommend a known Speed potion beforehand would
use information unavailable to the actor. Terminal reveal is explicitly marked
in the final anchor.

| Resource/event | Actual observation | Meaning and limit |
| --- | --- | --- |
| Starting cures | Two known Cure Serious Wounds; sold one each at271 and281, confirmed pack2→1→0. | A real earlier stock decision, not proof that a cure would rescue this later life. |
| Recall | Deposited at Home121; terminal Home still lists it. | Not carried or available as an immediate escape in the mold pocket. |
| Phase Door | Bought691, last scroll read14801, absent by14811. | No Phase at22774; not a life that never bought an escape. |
| Fuel | Six spare torches initially bought; three more around12661–12671. One spare equipped21084. | Nine spares at death, including the replaced3833-turn torch. Excess relative to immediate cures is an inventory-policy question, not a slot-capacity proof. |
| Learned prayers | Detect Evil learned at1; Call Light learned21074. Both cost1MP,6% native failure, Ready throughout the fatal loop. | No learned Minor Healing, Bless, Portal or offensive prayer. Ready is native admission, not Borg selection or success. |
| Mace | Equipped Mace(2d4)(+0,+0); native visible mold2HP, no attacks landed on it. | A landed ordinary physical hit has enough raw dice damage to be a credible kill candidate; hit probability, action timing and a successful branch remain untested. |

The historical `Studying spell/prayer Bless` note appears before learning
Detect Evil and Call Light. This is not evidence that Bless was learned or
that the replay omits healing: this Priest lacks CHOOSE_SPELLS, and native
[`do_cmd_study_book`](https://github.com/diegoami/borg-rs/blob/aa4ad6cfeea2cf19870e224219dfe66688175c6e/angband/src/cmd-obj.c#L1200)
selects a random eligible prayer from the chosen book. Call Light hurts
light-sensitive monsters; this mold has fire vulnerability, not light
vulnerability. Neither learned prayer is a direct solution to its melee.
Mace capability is not evidence that choosing combat over a safe retreat is
a general improvement. It already killed three wild dogs with the same mace
at21044–21064, but that is a different encounter, not a mold rescue test.

## Decision timeline

[decision-timeline.csv](decision-timeline.csv) retains39 actual frame boundaries,
including earlier stock changes, four staircase cycles, prayer prompts, rests
and damage acknowledgements. [21 original anchors](analysis-anchors/README.md)
keep full terminal screens, native known gear/prayers and native-visible
monsters. Key frames precede their queued key. Same-turn acknowledgement
frames are not additional tactical chances; world ticks are not all decisions.

| Actual turn/phase | Knowledge/resources | Chosen action/result | Alternative or question | Evidence/confidence |
| --- | --- | --- | --- | --- |
| 121 /271 /281 | Born with Recall and2CSW. | Store Recall; sell both cures. | Why does power/shop scoring prefer sale over a low-level reserve? | Original native pack transitions/messages; valuation not traced. |
| 691 /12661–12671 /14801 | Phase purchased; fuel rises6→9 spares. | Later spends only Phase while twitchy. | Retaining/replenishing escape versus existing town opportunities. | Actual purchase/consumption; not counterfactual restocking. |
| 21044–21084 | CL1→2, mace usable, two prayer slots learned by then. | Kills3 dogs; learns Call Light; replaces torch. | Chance-driven learning versus stock backup. | Original kills/learning/native learned list. |
| 21744–21774 | Full15HP, MP3→0. D1 scary-monster detections, D2 deemed wrong depth. | Four D1→D2→D1 cycles, then rest. | Earlier level-reroll/preparedness policy contributes exposure. | Key/depth sequence; no claim this alone is a defect. |
| 21774–22754 | Final D1 down stair(33,49), fullHP, mana0→4. | Rests, then Detect Evil fails; mana3. | Failed detection does not report mold; mold is not evil anyway. | Native prayer failure, no mold knowledge yet. |
| 22764 /22774 | Full15HP/3MP on stair; opens north door. Mold now visible2HP at(34,47). | Opens door then moves north. | Stay/close door/descend instead of first exposure. | Door command22764; result and first mold view22774. |
| 22784 | Doorway(33,48), HP9, MP3, computed danger16. | Caution chooses south desperate stair step. | Attack would require reaching later selection; it does not. | Qualified trace: caution returns before attack selector. |
| 22794 | On down stair, HP9, danger0, no flee/leave/scary/descend flag. | `dark-3` north, second exposure. | Descend/hold/close door. | Known doorway cache reused; no danger recomputation. |
| 22804→22814 | HP8, adjacent danger16; same resources. | South desperate step, then north exploration. | Preserve escape intent or invalidate exceptional-mode cache. | Same observed branch/cache pattern. |
| 22824→22834 | HP5, danger16 adjacent and0 on stair. | Third south step, then fourth north exposure. | Excessive-danger flee guard is blocked by ready_morgoth0. | Existing source156 gate observed relevant at16>2×5HP. |
| 22841 death | Doorway; HP5→2→-2, MP3; mold still2HP. | Two spore blows, death acknowledgements. | No living decision after terminal blow. | Final phase`death`, frame1556; no invented late action. |

### What the private trace establishes

The selected states in [private-gates.jsonl](private-gates.jsonl) bind these
mechanisms to this trajectory rather than merely citing possible code paths:

1. At22784/22804/22824, `fleeing=false`, `leaving=false`, `scary=false`,
   `stair_more=false`, `ready_morgoth=0`. Computed current danger is16, with
   avoidance9/8/5. [`desperate_for_stairs`](https://github.com/diegoami/borg-rs/blob/aa4ad6cfeea2cf19870e224219dfe66688175c6e/borg-rs/src/borg_caution.rs#L862)
   admits the nearby down stair. The flow temporarily enables `desperate`,
   queues south, sets goal type7, clears `desperate` and returns from caution.
   The dungeon loop's [later attack selection](https://github.com/diegoami/borg-rs/blob/aa4ad6cfeea2cf19870e224219dfe66688175c6e/borg-rs/src/borg_think_dungeon.rs#L1589)
   is never reached while adjacent in these three decisions. This explains
   the absence of fighting without inventing a losing attack score.
2. At22794/22814/22834, current danger is0 on the physical down stair,
   all those persistent flags remain false, and caution takes no action.
   `choose_stairs` is never called in the tail. The [descent block](https://github.com/diegoami/borg-rs/blob/aa4ad6cfeea2cf19870e224219dfe66688175c6e/borg-rs/src/borg_caution.rs#L1681)
   requires `stair_more`; arriving with a GOAL_FLEE flow does not itself set
   that flag. The stationary-monster [distance gate](https://github.com/diegoami/borg-rs/blob/aa4ad6cfeea2cf19870e224219dfe66688175c6e/borg-rs/src/borg_danger.rs#L2152)
   is consistent with the observed0 at distance2. The loop reaches attack
   selection there, where the mold is not adjacent, then later exploration.
3. During each desperate flow, the doorway starts `know=0,icky=0`; spread
   marks `know=1` while skipping danger because `desperate=true`. At the next
   normal [dark-3 spread](https://github.com/diegoami/borg-rs/blob/aa4ad6cfeea2cf19870e224219dfe66688175c6e/borg-rs/src/borg_flow_dark.rs#L751)
   that entry remains `know=1,icky=0`, despite `desperate=false`. The
   [once-per-cache danger gate](https://github.com/diegoami/borg-rs/blob/aa4ad6cfeea2cf19870e224219dfe66688175c6e/borg-rs/src/borg_flow.rs#L634)
   is therefore skipped. Normal fear cutoffs are2/2/1; the doorway cost becomes2
   and the committed exploration costs3. No new doorway-danger calculation
   occurs in those normal spreads. This directly supports cache reuse as a
   contributor to re-entry; clearing it or preserving flee intent has not been
   tested as a correction.
4. Existing [source156](https://github.com/diegoami/neo-angband-tracker/issues/156)
   concerns the [ready_morgoth==1 excessive-danger guard](https://github.com/diegoami/borg-rs/blob/aa4ad6cfeea2cf19870e224219dfe66688175c6e/borg-rs/src/borg_caution.rs#L1624).
   It cannot trigger on the first two returns:16>18 and16>16 are false. At
   HP5,16>10 is true but observed ready_morgoth0 rejects fleeing. This is a
   concrete relevant gate, not proof that its isolated correction rescues.
   Existing [source152](https://github.com/diegoami/neo-angband-tracker/issues/152)
   false tracked-stair flags do not alone explain a down-stair refuge at0
   local danger: physical FEAT_MORE is already tested. One down stair and no
   known up stairs are observed, so the farthest-stair scan defect is not
   isolated by this case.

## Brainstorming and candidate fit

| Candidate/explanation | Supporting evidence | Remaining uncertainty | Cost/possible harm | Next evidence / ownership |
| --- | --- | --- | --- | --- |
| Exceptional-mode danger cache reused by normal exploration | Exact doorway know0→1 in desperate flow, normal reuse1/icky0, no re-evaluation, three re-entries. | Initial unsafe door entry and whether this change alone restores useful progress. C source has the same cache/mode pattern, but no independent native C reproduction here. | Invalidating too broadly may cause stalls or hide a necessary escape through danger. | Priority1: small legal native C mode-transition scenario and controls through C consumer; no duplicate source defect claimed from Rust alone. |
| Emergency flow lacks persistent descent intent | Actual GOAL_FLEE7 after south step, persistent flags false, no choose_stairs call, exploration replaces goal. | Which intent should persist and when to release it; descending can meet worse danger. | Permanent flight could create stair churn or block exploration. | Separate explicit-descent/hold branches; retain failed outcomes and progress measure. |
| Existing excessive-danger guard156 blocks low-level flight | At22824 danger16>10, ready_morgoth0, fleeing false. | Correction interaction with cache/flow and generated D2 danger. | Fleeing at different times can waste resources or enter worse levels. | Existing canonical lead156; coordinate C owner and accepted milestone, not a duplicate issue or immediate Rust carry. |
| Caution preempts potentially lethal mace strike | All three adjacent tactical decisions return on stair movement before attack selection; mold2HP versus2d4 mace. | No actual attack-score or native hit outcome because selector was not reached; safe retreat is normally desirable. | A miss still permits two spore blows; forcing combat universally is unsafe. | Isolated one-strike diagnostic, separate from intent/cache experiments. |
| Birth-cure sale and survival stock | Native cures2→1→0 at271/281; Recall Home; Phase spent; no known cure/escape later; nine spares. | No exact shop candidate scores, available cheaper replacement or counterfactual resource consumption. | Retaining stock spends money/capacity and may delay growth; torches are useful. | Existing C41 reserve/180 identification leads; trace actual power/shop scoring first. |
| Missing learned healing / unlucky learning | Native learned only Detect Evil/Call Light; Priest study randomly selects from book. | Different legal learning order or spells cannot be assumed for the same RNG prefix. | Training/waiting for a preferred prayer may consume turns and risk. | No missing-view or Bless-learned bug established. Carrying consumable backup is separate policy. |
| Unidentified speed potion could help | Postmortem reveals Muddy=Speed. | Unknown during life; successful drinking/selection/rescue untested. | Unknown Cloudy was Blindness; blind use can worsen danger. | Preserve knowledge boundary; do not present as refusal of known Speed. |
| Replay or Rust-only translation failure | Dedicated qualification required because observer reads private fields. | Original engine rerun and observational build both preserve all declared outputs/saves; shared source patterns observed, no isolated translation fault. | Blaming replay or adopting a mismatched current engine corrupts causal inference. | Original evidence retained; next native reproduction under existing ownership. |

## Priorities and next evidence

1. **Next discriminating work is a bounded native reproduction/control of
   cache validity across desperate→normal flow**, plus an independently
   scoped reproduction of persistent stair intent. The actual Rust trace is
   now available; do not repeat the vague request for all private gates.
   Construct legal terrain/monster/stair input, preserve exception-mode escape,
   ordinary safe flow and blocked-route controls, compare exact resulting
   movement/state, and retain unfavorable outcomes. Coordinate through the
   independently owned C consumer. A shared source pattern and a fatal Rust
   trace do not by themselves authorize a source patch or new C bug claim.
2. Compare separately a specified descent at22794 (or later return), a hold/
   door-close alternative and a single mace attempt at an adjacent decision.
   An uninterrupted full-state branch is stronger than loading a native save;
   loading restores serialized engine state but starts new private Borg memory.
   Freeze source/origin/RNG/action timing and continuation limits. Score both
   survival and return/progress; no combined cache/intent/attack patch that hides
   which mechanism mattered. No such branch was run in this assessment.
3. Trace exact birth-CSW sale valuation and last-town fuel/cure/Recall choices.
   Historical [power scoring](https://github.com/diegoami/borg-rs/blob/aa4ad6cfeea2cf19870e224219dfe66688175c6e/borg-rs/src/borg_power.rs#L1417)
   rewards CSW in that ordinary block only above maxCL10, while low-level CLW
   has a separate reward; that is source context, not a recorded sale-score
   explanation. Keep this question separate from tactical loop correction and
   from simply declaring torches excessive. Related C41/180 retain ownership.
4. Use the [accepted C-milestone contract](../../C-CONSUMER-ADOPTION.md) for
   any shared correction. M5 stays0203227, no first C gameplay target is
   selected, and this analysis adopts no strategy. A rescued replay would
   still require prospective independent survival and progress evidence before
   a policy-benefit claim.

## Outcome and limits

The owner's loop/no-fight/no-heal/many-torches observations are supported.
The stair direction, spore mechanism and resource history are now precise:
downstairs to D2; stationary adjacent melee; birth cures sold, Recall at Home,
last Phase spent; no learned healing. The mace is a credible kill candidate,
but no successful strike or safe descent is demonstrated.

The read-only trace establishes why attacks are not reached while adjacent,
why descent is not selected on the refuge, and how exceptional-mode cache
reuse admits normal exploration back into the danger. The minimal fix, native
C reproduction, alternate-action survival and population benefit remain open
on [#5](https://github.com/diegoami/borg-rs/issues/5). Original record/build/
observations are immutable; this remains one collected life. No gameplay fix,
C milestone adoption or owner review mark follows from this assessment.
