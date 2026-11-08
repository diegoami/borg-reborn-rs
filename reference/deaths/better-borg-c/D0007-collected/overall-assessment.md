# D0007: early exposure, repeated displacement, then unused caster resources

## Overall picture

This is the **C42 Gnome Mage, character level4 on dungeon level1**, not a
level1 character: source22f817c96843, seed2624955988, soldier death179053,
HP-5/27 and SP2/8. The original build/start/options identity is unchanged in
record.json/build.json. [Owner review](https://github.com/diegoami/better-borg-c/issues/163#issuecomment-6712916845)
remains on163. [Complete unchanged-engine verification](review-verification.json)
passes all recorded inputs/actions/clock/files/native saves/state/terminal frames;
this verification is not a new independent death.

The consequential sequence begins before the final fight. After returning to
known upstairs it collects room loot despite a town-flight objective; visible
threats include a cutpurse, soldier, salamander and white jelly. It phases from an
adjacent clear icky thing at full HP, returns along its known upstairs flow,
then repeatedly spends escape resources. The soldier wakes while it is still
trying to get back. Final melee against the jelly leaves it poisoned and lets
the soldier close. The immediate killer is the soldier; premature exposure,
return/loot priorities, movement gates and recovery eligibility are competing
contributing mechanisms, not established defects.

This was not an empty-resource death: five known Cure Light Wounds and2MP remain.
The native spell display says Phase Door costs2MP, failure8%, Ready. Borg
eligibility, landing safety and healing call order remain unobserved. It has no
carried Recall. Equipment is a plain rapier, robe, small metal shield[5,+4], iron
helm, gloves, cloak and sandals[1,+4]; no launcher. First Spells is carried.
Late pack has11stacks, so a full pack is not an established explanation.
No confusion/blindness/amnesia is recorded in the late native timed effects;
poison is present after jelly contact. Actual hidden item/monster fields in
snapshots are diagnostic information and must not become extra Borg knowledge.

## Decision timeline

[Exact native anchors](review-anchors.json), [Borg notes](review-borg.log),
[actions](review-actions.txt) accompany the original phase/frame boundaries. Borg text has turn stamps only: it cannot establish intra-turn
ordering against an arbitrary same-turn terminal frame. The anchors select the
last recorded frame at each requested turn. Engine ticks are not all decisions.

| Actual turn/phase | Known state/resources | Chosen behavior | Alternative to examine | Evidence/limits |
| --- | --- | --- | --- | --- |
| 178286 command, frame193452 | Final stair position(102,3),27HP/8MP; town-flight flag1 | Leaves stair toward loot | Return to town before this room | Several down/up/town/down transitions share178286; do not collapse them into one level entry |
| 178325 tick, frame193497 | (102,7),27HP/8MP; visible centipede, cutpurse, soldier, salamander, jelly | Continues item flow; food/Boldness collected | Abandon loot and use known upstairs while room quiet | No sleep field for each enemy; owner’s “everybody sleeping” is not proved |
| 178369–178379 command, frame193569 | Stair cost7 at178369; then(100,9), clear icky adjacent,27HP/8MP, danger23 | First Phase Door is a **spell**, SP falls8→6 | Compare fighting single pursuer and early walking escape | Trace reports unsafe samples then acceptable Phase; no proved superior fight |
| 178390 tick | (109,10),27HP/6MP, no visible enemy | Follows upstairs costs12→7 before renewed contact | Assess alternate route/avoid room | Return into room is on a known stair route, not proof of aimless exploration |
| 178449 /529 /640 /740 /891 | Cutpurse contact; five initially carried Phase scrolls | Reads all five; another Phase spell178811 | Earlier route commitment/resource reserve | Last scroll178891; repeated displacement changes route and RNG |
| 178968 tick, frame194306 | First native “The soldier wakes up.” message | Continues room/stair sequence | Earlier withdrawal/recovery | Owner approximate178988 corrected by original visible message |
| 178972–178999 tick, frame194345 | Stair flow cost8→6;27HP/2MP; player(100,8) | Attempts upstairs flow before final contact | Preserve this objective through contact | There **are** recorded walking retreat attempts; avoid saying it never tries |
| 179002 command, frame194352 | HP25/27,SP2/8,poison5; jelly north(100,7), soldier(99,11), cutpurse(100,10); stairs(102,3) | Rapier attack jelly | NE(101,7) visible unoccupied floor; native-ready Phase or recovery | North blocked, NE not occupied; route legality/outcome and actual Borg admission need native comparison |
| 179012 /022 /032 /042 /052 | HP20/18/14/10/2; poison accumulates, soldier becomes adjacent south179022;2MP/five cures persist | Repeated melee; no Phase/heal observed | Earlier escape/heal before soldier closes | No proved safe last-second rescue; death179053 |

## Brainstorming and candidate fit

| Candidate/explanation | Support | Uncertainty/competition | Cost/harm | Linked next evidence |
| --- | --- | --- | --- | --- |
| Town-return objective versus nearby loot | Captured fleeing_to_town1; item collection before escape. Historical think-dungeon calls stair-scum before normal caution atdepth1; helper offers GOAL_TAKE/nearby loot first | Call-order source supports mechanism, not actual branch attribution at each frame; town also contains scary enemies | Strict loot leash reduces growth/profit; repeated town escape may oscillate | [159](https://github.com/diegoami/better-borg-c/issues/159): observe helper/goal transitions before178325 |
| Phase propensity versus single-enemy fight | First cast atfullHP27 adjacent clear icky, danger23; displacement lengthens return | FullHP does not mean safe; danger includes status/economic penalties and potential combat; no counterfactual proved | Less phase can increase melee/status death; more phase spends mana and relocates unpredictably |159: compare actual legal route and single-enemy fight separately, retaining failed branches |
| Walking around jelly/upstairs | Known stair and NE floor; existing retreat flow precedes melee | Historical low-level scary stair-flow rejects faster adjacent monsters; jelly native speed121. Exact guard firing unobserved | Walking may incur attacks and intersect soldier; map route is not a successful escape |159: qualify179002 NE/upstairs diagnostic branch against remaining in melee |
| Native-ready Phase rejected or bypassed |2MP, carried spellbook, learned2MP/8% spell remainsReady; no late cast | Native readiness is not Borg eligibility. Own traits, restriction/curse checks, phase safety samples, failure thresholds and call priority need observation | Forcing phase may land worse or fail; changing several gates hides attribution |[193](https://github.com/diegoami/better-borg-c/issues/193): inspect exact eligibility/rejection before gameplay change |
| Healing bypass while fleeing | Five known CLW unchanged; captured fleeing1. Historical caution wraps healing/defense in `!borg.goal.fleeing` | No proof that call/guard caused final decision or potion timing rescues | Healing consumes action while soldier closes; keeping a fleeing flag can be intentional |[34](https://github.com/diegoami/better-borg-c/issues/34): instrument call entry/guard and score, shared with previous deaths |
| Preparedness/restock | Mage has escapes, cures and book; no Recall/launcher; plenty of oil/food | Not evidence that stock shortage caused death; stock exists but is unused. No need duplicate181/183 for this case now | More shopping/weight delays progress; forcing Recall atCL4 may be costly |Defer new inventory proposal until eligibility/route mechanisms separated |
| Bad luck rather than defect | Random Phase placement, wake timing, poison and melee rolls contribute | Exact original is reproducible, alternative actions diverge RNG; no population claim from one death | Overfitting one rescue may harm other classes/cases |Keep baseline; prospective bounded survival/progress evaluation after a justified change |

## Priorities and next evidence

The owner clarified after completing this replay review: **leave the crowded
room early rather than continue fighting with whatever resources remain; there
are too many enemies.** [Recorded feedback](https://github.com/diegoami/better-borg-c/issues/163#issuecomment-6713938482).
This makes early disengagement at178325 and commitment to the known upstairs
route the central policy question under159. Unused Phase/healing under193/34
are supporting investigations, not substitutes for avoiding the crowded fight.
The recording shows stair-return attempts; why they are interrupted remains
unresolved. An earlier retreat has not been demonstrated to survive, and the
owner’s completed review remains complete.

First inspect193’s native/Borg mismatch and shared34 recovery call order at
179002/179022 using the pinned historical start and a diagnostic-only observer.
Retain prefix agreement, actual eligible/rejected reasons and unsuccessful
comparisons. Source Mage6MP reserve is cleared belowCL35, so that reserve alone
is not a valid explanation for thisCL4 mage. The strict emergency HP threshold
`hp < max_hp/10` also does not by itself explain the entire earlier sequence.
Then examine159’s early stair-scum loot objective and a legal NE/upstairs
attempt; these are distinct from last-second phase/healing. No new strategy,
threshold or claimed fix is adopted by this review. Any reusable C defect must
first be reported to canonical neo-angband-tracker/source207; local policy
candidates follow the separate [implementation/evaluation gate](../../measurement/DEATH-IMPROVEMENT-WORKFLOW.md).

## Outcome and limits

The owner’s early-retreat and late-resource concerns are well grounded. The
recording additionally shows repeated stair-return attempts, so “no plan” is too
strong; the unresolved question is why item/escape/combat priorities defeat or
interrupt that objective. It proves neither that the first Phase was wrong nor
that any proposed bypass/Phase/heal would survive.163 feedback is processed,
193/159/34 remain open, and review completion does not resolve those actions.
