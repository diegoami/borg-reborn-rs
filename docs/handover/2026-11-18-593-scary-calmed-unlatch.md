## 2026-11-18: the scary flag clears when the scary monster is hurt or afraid (#593)

Owner, the sibling of #590. #539 makes a hurt or afraid named-list or damage-rule monster no longer scary in the scan, but with the shipped latch a flag already set stays set: hurting Grip, or making it flee, does not end the flight.

- `caution_scary_unlatch_calmed` (0, off in `harness/borg.txt`): 1 clears the flag once no scary monster is known and a monster of a race that made the level scary (`mon.scary_races`, #590) is known hurt or afraid in this scan (`mon.scary_calmed`, from `classify_scare`'s `calmed_scare`), with the latch on or off. If the monster recovers, the next scan finds it scary again.
- `unlatch_when` now takes the three settings (`UnlatchRules`): with the kill or the calmed rule on, only a kill or a calmed monster clears; with neither, the latch decides as before.

Stacked on #590 (#592). Left: a zoo arm beside #590's.

Review of #595:
- the race of a calmed scary monster is recorded too (`scary_races`), so a monster already hurt or afraid when first seen counts; before, the rule missed more clears than it made (the reviewer's instrumented count: 14, 3 and 46 missed against 4, 2 and 4 made, one on a scary level with Farmer Maggot). #590's kill rule now also matches such a race;
- with the calmed rule on, a kill clears the flag too (the stronger signal); with only the kill rule on, a calmed monster does not;
- docs: the thresholds are below the setting in `borg.txt`; resist-based scares are never calmed.
