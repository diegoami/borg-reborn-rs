# D0008: emergency stair retreat followed by repeated grey-mold re-exposure

## Overall picture

**Human Priest, character level1, dungeon level1**, seed1668846659;
source5c23742ced30ad76ab9778cfa8bf23f04fc0eb1f (C46). The grey mold kills at10940.
[Owner feedback](https://github.com/diegoami/better-borg-c/issues/163#issuecomment-6714388294)
is now tracked in [consumer212](https://github.com/diegoami/better-borg-c/issues/212).
The original record/build/input/map evidence retains its actual historical identity.
[Full unchanged-engine replay](review-verification.json) is VERIFIED; a separately
built [read-only C diagnostic replay](observer-verification.json) also matches all
original frames, actions, input/clock/file channels, trace and native save bytes.
Neither rerun is an additional independent life.

The owner correctly identified no fighting in the fatal encounter. The priest
is **moving**, rather than stuck on one tile: from the threatened square(54,7)
it repeatedly retreats west onto the known upstairs(53,7), then explores east
back into the stationary mold's adjacent melee reach. Three Cure Light Wounds
extend the loop; it never takes the upstairs or attacks the mold. The mold's
"spores" are adjacent melee, not a ranged spore spell.

The qualified C trace independently reproduces the cache interaction reported in
[the owner's Rust reference](https://github.com/diegoami/borg-rs/issues/5): emergency
flow marks a square checked while omitting its danger calculation; later normal
exploration reuses that entry without recalculation. The C flow module is
byte-identical to pristine e473720. The reusable mechanism is reported first as
[source404](https://github.com/diegoami/neo-angband-tracker/issues/404), with source
ownership and a narrow correction/control matrix still pending. No strategy,
fix or claimed rescue is implemented by this review.

## Decision timeline

[15 original frame anchors](review-anchors.json), [Borg notes](review-borg.log),
[queued keys](review-actions.txt) and [qualified private trace](private-trace.log)
retain actual boundaries. Anchors select the last original frame at each turn;
Borg notes have turn stamps only, so do not imply intra-turn ordering against an
arbitrary same-turn frame. Engine ticks are not independent Borg choices.

| Turn / actual phase | Knowledge/resources | Action / result | Alternative / qualification |
| --- | --- | --- | --- |
| 9797–9807 | Town down stair; Bless cast before descent | Enters D1 and rests on known upstairs, replenishing mana | An ordinary legal descent, not a trap or unexpected teleport |
| 10807 command, frame11851 | Upstairs(53,7),13HP/2MP,3CLW,2Phase | dark-2 queues east to(54,7); original flow evaluates that grid danger0 | Mold was not yet known; initial exposure is not established cache pollution |
| 10817 command, frame11865 | Adjacent mold(55,8),6HP, danger16 | First CLW; no attack selection reached | Melee, escape or retreat are alternatives, not guaranteed wins |
| 10827 command, frame11879 | 7HP, danger16 | Desperate stair flow queues west; temporary desperate flag cleared afterward | Caution returns before normal attack selection |
| 10837 command, frame11891 | Back on upstairs,7HP; local danger0, flee/leave/scary/stair_less0 | Attack selection is reached from this nonadjacent square; dark-3 queues east | Taking the known upstairs is available; arrival has not established ascent intent |
| 10847 command, frame11905 | Back adjacent,2HP/2MP | Second CLW, recovers to13HP by10857 | Healing works; it does not resolve the exposure loop |
| 10857→10867 command, frames11919/11931 | 13HP at threatened tile, then upstairs | Desperate west retreat, then east exploration again | Same cache transition, not a new independent encounter |
| 10877 command, frame11945 | Adjacent,8HP | Third/final CLW; later11HP | Two Phase scrolls remain; no healing prayer learned |
| 10887→10897 command, frames11959/11971 | 11HP; retreat reaches upstairs | West retreat, then east again | Stair route is physically reachable and actually used |
| 10907→10917 command, frames11985/11997 | 8HP, no CLW | West retreat, then east exploration | Lack of healing does not cause recognition or ascent commitment |
| 10927→10937 command, frames12011/12023 | 2HP,2MP,2Phase; returns to upstairs | West retreat, then last east step | Source156 ready_morgoth guard is a contributing candidate; no successful escape tested |
| 10940 death | Native grey-mold melee | Priest dies; mold remains2HP in retained diagnostics | No melee attack occurred in this tail; no counterfactual outcome claimed |

## Preparedness, prayers and legal options

The character has a Mace(2d4)(+0,+0), cloak[1,+0], gauntlets[3,+2], lit torch,
no body armour or launcher. It carries7food rations,10spare torches, a prayer
book, initially3knownCLW and2knownPhase. Pack pressure is not supported: the
pack has5stacks before cures run out. Two Phase scrolls remain at the death;
this is not a complete escape-stock exhaustion case. Recall is not carried.
Only **Bless** is learned (2MP/14% native failure, Ready in the encounter); there
is no learned healing, attack or escape prayer. Recommending a healing prayer
would require a capability the character does not have.

At(54,7) the grey mold is diagonally adjacent at(55,8). A normal weapon attack is
an available attempt, but a hit/kill is not guaranteed. West to upstairs is not
merely an imagined route: the recording performs it five times. Taking '<' on
that staircase is the immediate town-return attempt; its continuation/survival
was not tested. Phase is available stock, but its Borg admission/safety samples
and actual destination outcomes are not in this observation. Do not force a
spell/Phase or increase aggression before isolating the observed control fault.

## Qualified private mechanism

[Observer provenance](observer-provenance.json) compares all1569historical source
files: exactly three differ, solely by the retained stderr reads in
[observer.patch](observer.patch). No added danger evaluation, RNG draw, branch
change or cache write is introduced. The original engine and observed build
both reproduce the complete captured death, not merely an outcome/seed match.

At10827/10857/10887/10907/10927, desperate flow clears the pending danger cache,
then marks(54,7) `know=1,icky=0` while `desperate=1` bypasses danger calculation.
At10837/10867/10897/10917/10937, normal flow sees `wipe=0,desperate=0,know=1,icky=0`
and accepts that same tile without the guarded evaluation. This proves reuse of
an entry that was not danger-checked in the previous mode. It does **not** give
a new normal-mode danger value at those later points: none was calculated.

The ordinary caution observations at the upstairs are danger0, fleeing0,
leaving0, scary0, stair_less0, ready_morgoth0. Desperate movement is a one-step
response and returns before attack selection while adjacent. On the safe stair,
normal processing resumes without a persistent ascend objective. The source
RF_NEVER_MOVE danger rule removes stationary-monster melee beyond adjacency;
loss of LOS/remembered-monster state also needs assessment before interpreting
what a future invalidation would recalculate. Do not feed hidden monster data
from the diagnostic snapshots into the Borg's decisions.

## Brainstorming and candidate fit

| Candidate / explanation | Evidence and fit | Uncertainty / costs | Action |
| --- | --- | --- | --- |
| Exception-mode flow cache reused in normal exploration | Qualified C trace proves know0→1 without danger evaluation, followed by reuse and east re-exposure; pristine flow module unchanged | Minimal isolated fixture and narrow fix/control matrix pending; later normal danger was never evaluated; frequent global clears can increase work | Source404, consumer212: qualify cache entries by evaluated mode or test scoped invalidation, preserving emergency retreat and safe-flow controls |
| No persistent escape/ascend intent | Reaches the known upstairs five times; local danger0 and escape flags0, then explores | Blanket staircase retreat or permanent flee intent can cause oscillation or suppress progress | Consumer212/shared159: explain temporary desperate response versus actual stair-taking, test ordinary '<' separately |
| Excessive-danger fleeing guard | At10927 danger16,HP2,ready_morgoth0; source156's guard rejects0 | Healing returns earlier at other low-HP decisions; other gates and a successful correction are not proved by this trace | [Source156](https://github.com/diegoami/neo-angband-tracker/issues/156): low-HP/native guard fixture, separately from cache fix |
| Always-false tracked-stair flags | Source152 exists unchanged and logs recognize this upstairs | At the refuge danger0, so changing on_upstair alone does not establish the required danger conjunction or escape intent | [Source152](https://github.com/diegoami/neo-angband-tracker/issues/152): keep separate; do not claim it explains this whole loop |
| Fight the mold instead | Equipped mace, diagonal adjacency, mold remains2HP; attack phase not reached while adjacent | Combat hit/damage and resulting RNG differ; increasing aggression may worsen other fights | Consumer212: legal melee diagnostic branch after original prefix, retaining failures |
| Phase/heal/restock | Three CLW used and2Phase unused; only Bless learned | More cures alone can prolong cycling; no prayer heal exists; Phase admission and landing safety unknown | Consumer212: escape eligibility after route/cache evidence; no new reserve proposal needed for this case |
| UI/harness fault or random bad luck | Full recording and original/observed reruns exact; native movement keys and HP loss are real | Random damage affects death timing, not the demonstrated repeated control/cache transition | Reject 'stood motionless/headless stalled' explanation; do not claim population benefit from one diagnostic life |

## Relation to the Rust case

The owner's reference is a **different independent life**: Rust D0006, Half-Elf
PriestCL2, seed2647001198, death22841, down-stair refuge and no carried known
healing/escape stock. Its qualified private trace reports the same
emergency-cache/normal-exploration reuse, and its attack selection is likewise
pre-empted by caution while adjacent. C D0008 independently supplies that
mechanism with an **upstairs leading toward town** and unused Phase stock.
Linking the reports supports shared-mechanism investigation; it does not identify
the seeds, traces, equipment, escape direction or future RNG as identical.
No Rust implementation or change to its frozen milestone is part of this task.

## Priorities and next evidence

First give source404 the native mode-transition fixture and controls, including
safe normal flow, hazardous normal flow, blocked routes and successful emergency
retreat. Inspect current remembered-monster state when leaving LOS as a separate
input to any recalculation. Then test narrow cache corrections and persistent
stair intent independently, with source156's low-HP guard and Phase admission
as distinct candidates. Any adopted correction needs the source proposal,
consumer carry, unchanged fail-before/pass-after and negative controls before
prospective survival **and** long-term progress evaluation. No strategy changes
are authorised by completing this review.

## Outcome and limits

Owner review is complete and recorded in the replay queue. The reported behavior
is now a reproduced native control/cache defect candidate with a directly observed
shared mechanism, rather than merely a generic poor-combat-policy complaint.
The original build/evidence stays frozen. Source404/156 and consumer212/shared159
remain unresolved: no correction, successful rescue or mortality benefit is
claimed. Review completion does not close those technical investigations.

## Later consumer correction (original review/evidence unchanged)

Source404 correction b71b32c152 is now verified for consumer212 on the current
48-carry core, with unchanged fixture after30/30, partial-fix controls and154prior
regression expectations. [Consumer verification](../../measurement/FLOW-DANGER-CACHE.md)
records an exact original replay with the current unmodified engine and a
separate corrected diagnostic: first difference10837, alive200000/CL5/HP56,
existing bravery/Phase fallback rather than stairs. This qualifies a narrow
cache correction; source152/156, escape intent and population benefit remain
separate. The original C46 build, maps, input, recording and owner review retain
their historical identity. Integration status is live on212.
