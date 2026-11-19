## 2026-11-19: no rest while breeders are known (#644)

Owner, on #615: "no rest if aware of any breeders, clear the breeders or
leave the level asap".

- **Done:** `recover_rest_breeders` (flag, compiled and shipped 0). While a
  breeder is known on the level, no rod, HP/SP or mana rest; the Borg hunts
  them when #558's hunt applies, otherwise starts leaving the level. Kept
  when confused, blind, hallucinating or stunned. Details in
  docs/CALIBRATION.md.
- **Checked** (after the review of #656, main `592c6984`): off plays main's
  keys in 18 of 18 classes-18 starts. On: no death in either arm, sum of CL
  100 against 97, 561 held rests in 13 games (126 hunting, 25 leaves, one
  hunt resumed after the rule's own leave). Unit tests
  `no_rest_while_breeders_are_known` and
  `the_hunt_resumes_after_the_rules_own_leave`.
- **Review of #656, fixed:** the rule's own leave stopped the hunt for the
  rest of the level (`breeder_engaged` reads `goal.leaving`); the hunt is now
  judged without it and clears it when it applies again. The decision and
  its effect are separate functions. `mon.breeders` replaces a second scan.
- **Not checked:** the breeder case itself. Priest 592068863 (Cemetery
  D-5153ba00…) died among breeders only in #616's leave arm; here it plays
  main's keys with the rule on (no rest came due while breeders were known).
  A zoo pilot.
- **Owner:** the shipped value, after a pilot.
