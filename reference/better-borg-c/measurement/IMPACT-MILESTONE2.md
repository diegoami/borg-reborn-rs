# Native C impact v2 — accumulated review package

[Milestone2](https://github.com/diegoami/better-borg-c/milestone/2) and
[review issue38](https://github.com/diegoami/better-borg-c/issues/38) pin the
seven-carry product at **2675d198e4c3fcb1c69ee63ef25a2b6ac27ab96f**. The original
pristine baseline remains **e473720ed88a0f0c7ca66815cef3000eb4a38bc3**. External
review has not occurred. The owner directs agent review/normal merge/measurement
between accumulated milestones; this package does not add a per-PR review stop.
Original [v1](MILESTONE-1.md) and its snapshot remain immutable.

## Independent corrections and controls

Exact source/applied commits, original credit and patch removal units are in
[series.json](../../patches/series.json), with a separate consumer issue per fix.
Canonical reusable C defects/proposals remain in neo-angband-tracker. All seven
are locally carried/open and patch-active; none is called upstream resolved.

| Source / consumer / merge PR | Demonstrated native effect | Proof and limits |
| --- | --- | --- |
| [82](https://github.com/diegoami/neo-angband-tracker/issues/82) / [4](https://github.com/diegoami/better-borg-c/issues/4) / [6](https://github.com/diegoami/better-borg-c/pull/6) | Safer available emergency swap; two-threat loss113→59HP | [9 cases,4 unchanged controls](SAFER-SWAP.md); constructed depth50 combat |
| [14](https://github.com/diegoami/neo-angband-tracker/issues/14) / [7](https://github.com/diegoami/better-borg-c/issues/7) / [10](https://github.com/diegoami/better-borg-c/pull/10) | Post-knockout kill reaction0→1 | [4 cases,3 controls](KNOCKOUT.md); restored information, no proved bad action/death |
| [110](https://github.com/diegoami/neo-angband-tracker/issues/110) / [15](https://github.com/diegoami/better-borg-c/issues/15) / [16](https://github.com/diegoami/better-borg-c/pull/16) | Remove extra20mana/100energy regeneration cast | [9 cases,7 controls](PREPARATION.md); extra duration tradeoff retained |
| [26](https://github.com/diegoami/neo-angband-tracker/issues/26) / [17](https://github.com/diegoami/better-borg-c/issues/17) / [18](https://github.com/diegoami/better-borg-c/pull/18) | Read cold protection for cold damage | [6 cases,4 controls](COLD-PROTECTION.md); acid-only21→0HP for2mana, reverse0→20HP for saved2mana |
| [78](https://github.com/diegoami/neo-angband-tracker/issues/78) / [20](https://github.com/diegoami/better-borg-c/issues/20) / [21](https://github.com/diegoami/better-borg-c/pull/21) | Ready left fire ring kills70HP ogre | [7 cases,5 controls](BOTH-RINGS.md); both single-loop alternatives fail, no full fight |
| [77](https://github.com/diegoami/neo-angband-tracker/issues/77) / [31](https://github.com/diegoami/better-borg-c/issues/31) / [32](https://github.com/diegoami/better-borg-c/pull/32) | Long-gap/first-use CCW heals60HP/clears confusion | [9 cases,5 controls](POTION-TIMING.md); recent-drink conservation leaves injury untreated; incomplete reversal fails sentinel |
| [102](https://github.com/diegoami/neo-angband-tracker/issues/102) / [36](https://github.com/diegoami/better-borg-c/issues/36) / [37](https://github.com/diegoami/better-borg-c/pull/37) | Recognize visible Itangast; retain one True Seeing potion | [5 cases,4 controls](ITANGAST.md); actual unseen names retained, protected setup, no damage/death |

All49 corrected native scenarios passed on the seven-carry core; all44 earlier
complete case logs match their preceding corrected results. Normal native gate
965/965unit+5/5frontend and29 tooling controls passed. Exact PR37 head
d89a2df123dc49cf4c7aab0d0749978092d4d0e0 hosted42221586833 was successful,
actual numerical logs inspected. Patch round-trips retain independent removal
units; no assertion, seed, damage/rule or unfavorable control was weakened.

## Measured gameplay and deaths

Frozen v1 n=3 remains seeds11/29/47,ordinary Human Warrior/default birth,
cap20000/wall120/stall30. Up to five carries the traces match pristine. Potion
timing changes private Borg RNG consumption and later behavior: all three still
survive the cap, but seed47 reaches level2 versus4 at unchanged maxdepth2.
That unfavorable result remains visible. Actual seven-carry merge was rebuilt;
its starts/full traces match six-carry45eac63bd exactly, with levels3/3/2 and
maxdepth1/3/2. [Every latest run/control summary](itangast-v1.json). No shallow
cohort survival benefit, runtime speedup or general no-regression claim.

[Deep v2](DEEP-IMPACT-V2.md), prospectively declared on33, measures the actual
five/six-carry builds only: n=3 paired starts,2deaths/1cap versus1death/2caps,
no stalls/timeouts/crashes/harness failures. Fixed11 survives longer with a
different trajectory; fixed47 has a higher level but shallower maxdepth. There
is no demonstrated population survival gain or causal same-encounter rescue.
Fixed11 cap/fixed29death full repeats match; fixed47 only first20000 observations
repeat. Later seven-carry code does not relabel these six-carry measurements.

[D0001/D0002 registry](../deaths/README.md) records actual original versions,
build/binary/data/patch/RNG/options/input/calendar identity, selected exact native
logs/maps/windows, assessment confidence and action issues22/34. Two starting
families/eight death process attempts include repetitions/version references;
this is not eight independent lives or a mortality-rate cohort. Every observed
natural death in the batch is retained. Unused resources are not automatic bug
proof: final gallant-case CLW healing is ineligible under existing danger guards.
No strategy/threshold change was made from a death hypothesis.

Recent living saves actually load/advance as separate diagnostics; native saves
omit Borg memory and compact monsters, while death saves omit dungeon state.
No exact midrun continuation/reloadable postdeath encounter claim. Original-start
replay windows match prior observations. Calendar provenance is recorded for new
runs; legacy D0001 absence stays explicit, not backfilled. Seasonal native rules
limit cross-calendar guarantees. Turns, energy, Borg calls and wall time differ.

## Pipeline changes, recovery and review questions

Since v1: bug-first planPR14; death/delivery records/export/aggregatePR23;
bounded weekly source inventoryPR25 and its actual issue/PR classification bug
repairPR27; runtime calendar provenancePR29; known versus unidentified inventory
qualificationPR30; complete paired measurements/dossiersPR35. See the linked
issues/checkpoint1 for exact merge pins/actual receipts. Weekly issue24 remains
ongoing inventory plus deliberate semantic review, not automatic imports or a
better-Borg verdict. Donor vein lead source280 is not yet a proved defect.

Raw generated runs/saves/builds stay ignored. Representative complete archives
have per-member hashes/readback verification and persistent RECOVERY.txt; public
reports contain copyable pin-based commands. Receipts on issues31/33/36 identify
actual local archives, not hosted binary uploads or workstation/tmp-only paths.
No full disposable worktrees are used as archives. Earlier failures, bad observer
save-compaction experiments, startup and watchdog controls stay recorded.

External review should check corrections/control validity and adverse tradeoffs,
cumulative interactions, instrumentation/replay non-interference, failure/censoring
and sample accounting, death/build/calendar attribution and future upstream
patch retirement. Reusable original-C findings go to source207 before source
fix/adoption; investigations22/34 are provisional. More verified bugs take
priority over new exploratory runs. Official game tracking remains deliberate;
no official posts, tactics, Rust polling, private data or deployment is included.
