# Native death records

Bugs always take priority. Each natural death gets a curated dossier with its actual source/build, logs, evaluation, diagnosis and linked actions. Full runs/saves/builds stay outside Git. Replays append to the case; later fixes never overwrite its identity.

To comment, open **Discuss** beside the case below. The linked GitHub investigation is the durable feedback thread; suggestions are checked against evidence and linked to actions. See [the feedback process](FEEDBACK.md). Chat feedback is copied there before it is considered tracked.

Dossiers: 10; distinct starting conditions: 7; recorded death process attempts: 18 (including repeats/reference variants). This registry is not a survival-rate estimate.

| Case | Discussion | Actual source | Seed | Raw cause | Death turn | Diagnosis |
| --- | --- | --- | --- | --- | --- | --- |
| [D0001](D0001-stegocentipede/report.md) | [Discuss](https://github.com/diegoami/better-borg-c/issues/22) | `337e50eb6c73` | 11 | a stegocentipede | 174862 | original C behavior; defect not established |
| [D0002](D0002-gallant/report.md) | [Discuss](https://github.com/diegoami/better-borg-c/issues/34) | `45eac63bd9f7` | 29 | a gallant | 20445 | original C behavior; final healing ineligible under recorded danger, defect not established |
| [D0003](D0003-named-headless/report.md) | [Discuss](https://github.com/diegoami/better-borg-c/issues/34) | `127483bc4724` | 29 | a gallant | 20445 | D0002 reference replay on 29-carry build; defect not established |
| [D0004](D0004-named-headless/report.md) | [Discuss](https://github.com/diegoami/better-borg-c/issues/34) | `1b95154f65b2` | 29 | a gallant | 20445 | D0002 reference; verified headless recording/replay; defect not established |
| [D0005](D0005-visible-recording/report.md) | [Discuss](https://github.com/diegoami/better-borg-c/issues/34) | `1b95154f65b2` | 29 | a gallant | 20445 | D0002 reference; verified visible recording/replay; defect not established |
| [D0006](D0006-collected/report.md) | [Discuss](https://github.com/diegoami/better-borg-c/issues/177) | `2cebef1b0f39` | 1344203209 | a killer brown beetle | 73100 | policy/resource/escape investigation; native implementation defect not established; sidebar redraw mismatch confirmed |
| [D0007](D0007-collected/report.md) | [Discuss](https://github.com/diegoami/better-borg-c/issues/163) | `22f817c96843` | 2624955988 | a soldier | 179053 | policy/call-order/escape investigation; reusable C defect not established |
| [D0008](D0008-collected/report.md) | [Discuss](https://github.com/diegoami/better-borg-c/issues/163) | `5c23742ced30` | 1668846659 | a grey mold | 10940 | native emergency/normal flow cache and escape-intent investigation; original-C source404 reported |
| [D0009](D0009-collected/report.md) | [Discuss](https://github.com/diegoami/better-borg-c/issues/163) | `1cd426af4848` | 2817460426 | starvation | 218620 | verified starvation/resource-return failure; original-C quiver-capacity pickup defect reported as source419 |
| [D0010](D0010-collected/report.md) | [Discuss](https://github.com/diegoami/better-borg-c/issues/163) | `1cd426af4848` | 2060687716 | a kobold | 6309 | verified retreat/archery opportunity investigation; no reusable gameplay defect established |

## Aggregated actions

- [Observe actual existing speed-defense eligibility and p1/p2 in the pre-fight state](https://github.com/diegoami/better-borg-c/issues/22) — investigate before changing policy; cases D0001.
- [Trace last Home/town decisions and escape-stock constraints](https://github.com/diegoami/better-borg-c/issues/22) — investigate before changing policy; cases D0001.
- [Compare pristine upstream versus carried build](https://github.com/diegoami/better-borg-c/issues/22) — completed: identical death/trace; does not establish a defect; cases D0001.
- [Evaluate survival-resource reserves/restocking priority over spare equipment](https://github.com/diegoami/better-borg-c/issues/41) — recorded owner priority; proposal, not implemented; cases D0001.
- [Audit delayed Deep Descent escape from depth2/max7 and restock/preparedness before depth12](https://github.com/diegoami/better-borg-c/issues/22) — investigating recorded original-rule behavior; no defect established; cases D0001.
- [Observe existing heal eligibility/call order and fleeing flag before the fatal melee](https://github.com/diegoami/better-borg-c/issues/34) — investigate before changing policy; cases D0002, D0003, D0004, D0005, D0006, D0007.
- [Trace why Phase Door was exhausted and whether earlier escape was legally available](https://github.com/diegoami/better-borg-c/issues/34) — investigate before changing policy; cases D0002.
- [Compare pristine upstream versus carried build](https://github.com/diegoami/better-borg-c/issues/34) — completed: pristine/five-carry identical trace; same death on six-carry; no defect established; cases D0002.
- [Reconstruct earliest known crowd danger, entry and retreat/preparedness at level3](https://github.com/diegoami/better-borg-c/issues/34) — recorded owner observation; needs native causal investigation; cases D0002, D0003, D0004, D0005.
- [Evaluate first Phase Door20293 versus legal fight/walk alternatives and crowd retreat at next decision20303](https://github.com/diegoami/better-borg-c/issues/34) — investigating; no rescue or implementation defect established; cases D0002, D0003, D0004, D0005.
- [Evaluate early Phase Door/cure restock requirement linked to source370](https://github.com/diegoami/better-borg-c/issues/154) — proposed enhancement; no implementation or survival benefit established; cases D0002, D0003, D0004, D0005.
- [Evaluate theft danger contribution and sensitivity of first escape choice](https://github.com/diegoami/better-borg-c/issues/158) — proposed analysis; runtime decomposition and survival/progress benefit unproved; cases D0002, D0003, D0004, D0005.
- [Evaluate attempted door/upstairs retreat before repeated phases and after scroll exhaustion](https://github.com/diegoami/better-borg-c/issues/159) — proposed analysis; sleep timing, route knowledge and outcomes unproved; cases D0002, D0003, D0004, D0005, D0006, D0007.
- [Evaluate identification-driven inventory and town trading, auditing existing sell-ID behavior](https://github.com/diegoami/better-borg-c/issues/180) — proposed enhancement; priority/knowledge value and costs need evidence; cases D0006.
- [Evaluate ammunition reserves/spending, melee alternatives and combat preparedness](https://github.com/diegoami/better-borg-c/issues/181) — proposed analysis; no incorrect equipment choice or benefit proved; cases D0006.
- [Evaluate early Recall retention and destination-aware delayed Deep Descent escape](https://github.com/diegoami/better-borg-c/issues/183) — investigating; Recall at Home and two five-level drops observed; rescue untested; cases D0006.
- [Label native sidebar HP/SP redraw lag without modifying original game cells](https://github.com/diegoami/better-borg-c/issues/178) — confirmed presentation defect; viewer correction and native-frame/PTY controls pass; cases D0006.
- [Observe actual Borg spell/teleport/phase safety gates versus native readiness at179002/179022](https://github.com/diegoami/better-borg-c/issues/193) — needs evidence; original-C defect not established; cases D0007.
- [Narrow source404 correction verified on actual product with before/after and negative controls; track upstream acceptance and separate population evaluation](https://github.com/diegoami/better-borg-c/issues/212) — carried; product regression/controls verified; upstream acceptance and population benefit remain open; cases D0008.
- [Separate actual temporary desperate retreat from taking the known upstairs, plus melee/Phase admission; compare ordinary legal alternatives](https://github.com/diegoami/better-borg-c/issues/159) — investigating; no successful rescue established; cases D0008.
- [Audit ready_morgoth0 guard at lowHP separately from source152 flags and source404 cache reuse](https://github.com/diegoami/neo-angband-tracker/issues/156) — needs evidence; contributing candidate, not isolated fix proof; cases D0008.
- [Minimal quiver-capacity native fixture/correction with actual-room/stack/gold/ammo controls, then separate consumer carry](https://github.com/diegoami/better-borg-c/issues/235) — investigating; recorded native failure verified, fix not carried; cases D0009.
- [Earlier food budget/town-return and loot leash, accounting for REGEN and actual pack/quiver capacity](https://github.com/diegoami/better-borg-c/issues/41) — needs evidence; no policy change; cases D0009.
- [Identify useful carried stock and compare identification/shop priority without hidden item knowledge](https://github.com/diegoami/better-borg-c/issues/180) — proposed; shared investigation; cases D0009.
- [Recall storage/retention/reacquisition and available walking return before hunger becomes critical](https://github.com/diegoami/better-borg-c/issues/183) — needs evidence; no rescue claimed; cases D0009.
- [Unknown-edible eligibility for caster starvation at localdanger0, retaining harmful outcomes and resource costs](https://github.com/diegoami/better-borg-c/issues/235) — needs evidence; no forced use implemented; cases D0009.
- [Early town/upstairs commitment versus repeated normal desperate downstairs response after weakness](https://github.com/diegoami/better-borg-c/issues/159) — investigating; no guaranteed route asserted; cases D0009, D0010.
- [Inspect legal bow attacks, accepted staircase path and retreat-before-attack priority at6285/6295; account for later carries](https://github.com/diegoami/better-borg-c/issues/239) — investigating; no gameplay change; cases D0010.
- [Qualified native message/actual HP-loss/speed viewer channel](https://github.com/diegoami/better-borg-c/issues/240) — verified locally and hosted; delivery PR; cases D0010.
- [Replay valid maximized-window recordings through separately qualified reader; retain malformed-input rejection](https://github.com/diegoami/better-borg-c/issues/238) — verified; delivery PR240; cases D0010.
- [Known cure/escape preparedness before the dive](https://github.com/diegoami/better-borg-c/issues/41) — proposed; shared investigation; cases D0010.

Recurring mechanisms are grouped by canonical issue, not by number of replays. Investigations are hypotheses until native causal proof; original-C bugs route to neo-angband-tracker before source fix and consumer carry.

Create records with tools/record-death.py, curate assessment.json/report.md, then run tools/aggregate-deaths.py. Every report pins its actual build even after main changes.
