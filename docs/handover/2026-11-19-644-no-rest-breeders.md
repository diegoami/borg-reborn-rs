## 2026-11-19: no rest while breeders are known (#644)

Owner, on #615: "no rest if aware of any breeders, clear the breeders or
leave the level asap".

- **Done:** `recover_rest_breeders` (flag, compiled and shipped 0). While a
  breeder is known on the level, no rod, HP/SP or mana rest; the Borg hunts
  them when #558's hunt applies, otherwise starts leaving the level. Kept
  when confused, blind, hallucinating or stunned. Details in
  docs/CALIBRATION.md.
- **Checked:** off plays main `3ba17e2c`'s keys in 18 of 18 classes-18
  starts. On: the same one death, sum of CL 96 against 101 (mostly one
  Warrior, diverging early), 781 held rests and 20 leaves in 11 games. Unit
  test `no_rest_while_breeders_are_known`.
- **Not checked:** the breeder case itself. Priest 592068863 (Cemetery
  D-5153ba00…) died among breeders only in #616's leave arm; here it plays
  main's keys with the rule on (no rest came due while breeders were known).
  A zoo pilot.
- **Owner:** the shipped value, after a pilot.
